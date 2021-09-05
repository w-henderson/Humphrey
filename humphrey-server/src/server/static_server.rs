use humphrey::http::headers::ResponseHeader;
use humphrey::http::mime::MimeType;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;

#[cfg(feature = "plugins")]
use crate::plugins::manager::PluginManager;
#[cfg(feature = "plugins")]
use std::sync::Mutex;

use crate::cache::Cache;
use crate::config::{BlacklistMode, Config};
use crate::logger::Logger;
use crate::route::try_open_path;
use crate::server::proxy::pipe;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, RwLock};
use std::thread::spawn;

/// Represents the application state.
/// Includes the target directory, cache state, and the logger.
#[derive(Default)]
pub struct AppState {
    pub directory: String,
    pub cache_limit: usize,
    pub cache: RwLock<Cache>,
    pub websocket_proxy: Option<String>,
    pub blacklist: Vec<String>,
    pub blacklist_mode: BlacklistMode,
    pub logger: Logger,
    #[cfg(feature = "plugins")]
    plugin_manager: Mutex<PluginManager>,
}

impl From<&Config> for AppState {
    fn from(config: &Config) -> Self {
        Self {
            directory: config.directory.as_ref().unwrap().clone(),
            cache_limit: config.cache_limit,
            cache: RwLock::new(Cache::from(config)),
            websocket_proxy: config.websocket_proxy.clone(),
            blacklist: config.blacklist.clone(),
            blacklist_mode: config.blacklist_mode.clone(),
            logger: Logger::from(config),
            #[cfg(feature = "plugins")]
            plugin_manager: Mutex::new(PluginManager::default()),
        }
    }
}

/// Main function for the static server.
pub fn main(config: Config) {
    let app: App<AppState> = App::new()
        .with_state(AppState::from(&config))
        .with_connection_condition(verify_connection)
        .with_websocket_handler(websocket_handler)
        .with_route("/*", file_handler_wrapper);

    let addr = format!("{}:{}", config.address, config.port);

    let state = app.get_state();
    let logger = &state.logger;
    logger.info("Starting static server");

    #[cfg(feature = "plugins")]
    {
        let plugins_count = load_plugins(&config, state);
        logger.info(&format!("Loaded {} plugins", plugins_count));
    };

    logger.info(&format!("Running at {}", addr));
    logger.debug(&format!("Configuration: {:?}", &config));

    app.run(addr).unwrap();
}

/// Verifies that the client is allowed to connect by checking with the blacklist config.
fn verify_connection(stream: &mut TcpStream, state: Arc<AppState>) -> bool {
    if let Ok(address) = stream.peer_addr() {
        let address = address.ip().to_string();
        if state.blacklist_mode == BlacklistMode::Block && state.blacklist.contains(&address) {
            state.logger.warn(&format!(
                "{}: Blacklisted IP attempted to connect",
                &address
            ));
            return false;
        }
    } else {
        state.logger.warn("Corrupted stream attempted to connect");
        return false;
    }

    true
}

#[cfg(feature = "plugins")]
fn file_handler_wrapper(mut request: Request, state: Arc<AppState>) -> Response {
    let mut plugins = state.plugin_manager.lock().unwrap();
    plugins.on_request(&mut request, state.clone());

    let mut response = file_handler(request, state.clone());
    plugins.on_response(&mut response, state.clone());

    response
}

#[cfg(not(feature = "plugins"))]
fn file_handler_wrapper(request: Request, state: Arc<AppState>) -> Response {
    file_handler(request, state)
}

/// Request handler for every request.
/// Attempts to open a given file relative to the binary and returns error 404 if not found.
fn file_handler(request: Request, state: Arc<AppState>) -> Response {
    // Return error 403 if the address was blacklisted
    if state
        .blacklist
        .contains(&request.address.origin_addr.to_string())
    {
        state.logger.warn(&format!(
            "{}: Blacklisted IP attempted to request {}",
            request.address, request.uri
        ));
        return Response::new(StatusCode::Forbidden)
            .with_header(ResponseHeader::ContentType, "text/html".into())
            .with_bytes(b"<h1>403 Forbidden</h1>".to_vec())
            .with_request_compatibility(&request)
            .with_generated_headers();
    }

    let full_path = format!("{}{}", state.directory, request.uri);

    if state.cache_limit > 0 {
        let cache = state.cache.read().unwrap();
        if let Some(cached) = cache.get(&full_path) {
            state.logger.info(&format!(
                "{}: 200 OK (cached) {}",
                request.address, request.uri
            ));
            return Response::new(StatusCode::OK)
                .with_header(ResponseHeader::ContentType, cached.mime_type.into())
                .with_bytes(cached.data.clone())
                .with_request_compatibility(&request)
                .with_generated_headers();
        }
        drop(cache);
    }

    if let Some(mut located) = try_open_path(&full_path) {
        if located.was_redirected && request.uri.chars().last() != Some('/') {
            state.logger.info(&format!(
                "{}: 302 Moved Permanently {}",
                request.address, request.uri
            ));
            return Response::new(StatusCode::MovedPermanently)
                .with_header(ResponseHeader::Location, format!("{}/", &request.uri))
                .with_request_compatibility(&request)
                .with_generated_headers();
        }

        let file_extension = if located.was_redirected {
            "html"
        } else {
            request.uri.split(".").last().unwrap_or("")
        };
        let mime_type = MimeType::from_extension(file_extension);
        let mut contents: Vec<u8> = Vec::new();
        located.file.read_to_end(&mut contents).unwrap();

        if state.cache_limit >= contents.len() {
            let mut cache = state.cache.write().unwrap();
            cache.set(&full_path, contents.clone(), mime_type);
            state.logger.debug(&format!("Cached route {}", request.uri));
        } else if state.cache_limit > 0 {
            state
                .logger
                .warn(&format!("Couldn't cache, cache too small {}", request.uri));
        }

        state
            .logger
            .info(&format!("{}: 200 OK {}", request.address, request.uri));
        Response::new(StatusCode::OK)
            .with_header(ResponseHeader::ContentType, mime_type.into())
            .with_bytes(contents)
            .with_request_compatibility(&request)
            .with_generated_headers()
    } else {
        state.logger.warn(&format!(
            "{}: 404 Not Found {}",
            request.address, request.uri
        ));
        Response::new(StatusCode::NotFound)
            .with_header(ResponseHeader::ContentType, "text/html".into())
            .with_bytes(b"<h1>404 Not Found</h1>".to_vec())
            .with_request_compatibility(&request)
            .with_generated_headers()
    }
}

fn websocket_handler(request: Request, mut source: TcpStream, state: Arc<AppState>) {
    let source_addr = source.peer_addr().unwrap().to_string();

    if let Some(address) = &state.websocket_proxy {
        let bytes: Vec<u8> = request.into();

        if let Ok(mut destination) = TcpStream::connect(address) {
            // The target was successfully connected to

            destination.write(&bytes).unwrap();

            let mut source_clone = source.try_clone().unwrap();
            let mut destination_clone = destination.try_clone().unwrap();
            state.logger.info(&format!(
                "{}: WebSocket connected, proxying data",
                source_addr
            ));

            // Pipe data in both directions
            let forward = spawn(move || pipe(&mut source, &mut destination));
            let backward = spawn(move || pipe(&mut destination_clone, &mut source_clone));

            // Log any errors
            if let Err(_) = forward.join().unwrap() {
                state.logger.error(&format!(
                    "{}: Error proxying WebSocket from client to target, connection closed",
                    source_addr
                ));
            }
            if let Err(_) = backward.join().unwrap() {
                state.logger.error(&format!(
                    "{}: Error proxying WebSocket from target to client, connection closed",
                    source_addr
                ));
            }

            state.logger.info(&format!(
                "{}: WebSocket session complete, connection closed",
                source_addr
            ));
        } else {
            state
                .logger
                .error(&format!("{}: Could not connect to WebSocket", source_addr));
        }
    } else {
        state.logger.warn(&format!(
            "{}: WebSocket connection attempted but no handler provided",
            source_addr
        ))
    }
}

#[cfg(feature = "plugins")]
fn load_plugins(config: &Config, state: &Arc<AppState>) -> usize {
    let mut manager = state.plugin_manager.lock().unwrap();

    for path in &config.plugin_libraries {
        unsafe {
            match manager.load_plugin(path) {
                Ok(name) => {
                    state.logger.info(&format!("Loaded plugin {}", name));
                }
                Err(e) => {
                    state
                        .logger
                        .error(&format!("Could not initialise plugin from {}", path));
                    state.logger.error(&format!("Error message: {}", e));
                    state.logger.error("Ignoring this plugin");
                }
            }
        }
    }

    manager.plugin_count()
}
