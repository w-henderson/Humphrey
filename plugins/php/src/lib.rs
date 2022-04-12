use humphrey::http::headers::{HeaderType, Headers};
use humphrey::http::{Request, Response, StatusCode};
use humphrey::route::{try_find_path, LocatedPath};

use humphrey_server::config::extended_hashmap::ExtendedMap;
use humphrey_server::config::{RouteConfig, RouteType};
use humphrey_server::declare_plugin;
use humphrey_server::plugins::plugin::{Plugin, PluginLoadResult};
use humphrey_server::server::server::AppState;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::Write;
use std::net::{Shutdown, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::fcgi::record::FcgiRecord;
use crate::fcgi::request::FcgiRequest;
use crate::fcgi::types::FcgiType;

mod fcgi;

#[derive(Debug, Default)]
pub struct PhpPlugin {
    /// Acts as a thread pool of open streams to the interpreter.
    streams: Vec<Mutex<TcpStream>>,
    /// Keeps track of which streams are available.
    next_stream: AtomicUsize,
}

impl Plugin for PhpPlugin {
    fn name(&self) -> &'static str {
        "PHP Plugin"
    }

    fn on_load(
        &mut self,
        config: &HashMap<String, String>,
        state: Arc<AppState>,
    ) -> PluginLoadResult<(), &'static str> {
        // Parses the configuration
        let php_address = config.get_optional("address", "127.0.0.1".into());
        let php_port = config.get_optional("port", "9000".into());
        let php_target = format!("{}:{}", php_address, php_port);
        let stream_count = config.get_optional_parsed("threads", 8_usize, "");

        if let Ok(threads) = stream_count {
            // If the thread count could be parsed, start that many streams

            for _ in 0..threads {
                if let Ok(stream) = TcpStream::connect(&php_target) {
                    // If the stream successfully connected, add it to the list

                    self.streams.push(Mutex::new(stream));
                } else {
                    // Otherwise return a fatal error

                    return PluginLoadResult::Fatal("Could not connect to the PHP CGI server");
                }
            }

            state.logger.info(&format!(
                "PHP Plugin connected to FCGI server at {} with {} threads",
                php_target, threads
            ));

            PluginLoadResult::Ok(())
        } else {
            // If the configuration could not be parsed, return an error

            PluginLoadResult::Fatal("Could not parse the PHP plugin threads count")
        }
    }

    fn on_request(
        &self,
        request: &mut Request,
        state: Arc<AppState>,
        route: &RouteConfig,
    ) -> Option<Response> {
        if !request.uri.ends_with(".php") && !request.uri.ends_with('/') && !request.uri.is_empty()
        {
            return None;
        }

        match route.route_type {
            RouteType::File => {
                let path = PathBuf::from(route.path.as_ref().unwrap());
                let directory = path.parent().unwrap().to_str().unwrap();

                return self.inner_request_handler(request, state, path.clone(), directory);
            }
            RouteType::Directory => {
                let mut simplified_uri = request.uri.clone();

                for ch in route.matches.chars() {
                    if ch != '*' {
                        simplified_uri.remove(0);
                    } else {
                        break;
                    }
                }

                let directory = route.path.as_ref().unwrap();

                if let Some(located) = try_find_path(directory, &simplified_uri, &["index.php"]) {
                    return match located {
                        LocatedPath::File(path) => {
                            self.inner_request_handler(request, state, path, directory)
                        }
                        _ => None,
                    };
                }
            }
            _ => (),
        }

        None
    }

    fn on_unload(&mut self) {
        // Drain the streams iterator, shutting down every stream.
        for stream in self.streams.drain(..) {
            let stream = stream.lock().unwrap();
            stream.shutdown(Shutdown::Both).unwrap();
        }
    }
}

impl PhpPlugin {
    fn inner_request_handler(
        &self,
        request: &Request,
        state: Arc<AppState>,
        path: PathBuf,
        directory: &str,
    ) -> Option<Response> {
        // If the requested file is a PHP file, check that the file exists then process it
        let file_name = path.to_str().unwrap().to_string();
        if file_name.ends_with(".php") {
            // On Windows, paths generated in this way have "\\?\" at the start
            // This means the path length limit is bypassed
            #[cfg(windows)]
            let file_name = file_name.replace("\\\\?\\", "");

            // Insert the parameters into a hashmap
            let mut params: HashMap<String, String> = HashMap::new();
            params.insert("GATEWAY_INTERFACE".into(), "FastCGI/1.0".into());
            params.insert("REQUEST_METHOD".into(), request.method.to_string());
            params.insert("REQUEST_URI".into(), format!("/{}", request.uri));
            params.insert("SCRIPT_NAME".into(), format!("/{}", request.uri));
            params.insert("SCRIPT_FILENAME".into(), file_name);
            params.insert("QUERY_STRING".into(), request.query.clone());
            params.insert("SERVER_SOFTWARE".into(), "Humphrey".into());
            params.insert("REMOTE_ADDR".into(), "127.0.0.1".into());
            params.insert("SERVER_NAME".into(), "localhost".into());
            params.insert("SERVER_PORT".into(), "80".into());
            params.insert("SERVER_PROTOCOL".into(), "HTTP/1.1".into());
            params.insert("PHP_SELF".into(), format!("/{}", request.uri));
            params.insert("DOCUMENT_ROOT".into(), directory.into());
            params.insert(
                "CONTENT_LENGTH".into(),
                request
                    .content
                    .as_ref()
                    .map(|v| v.len())
                    .unwrap_or(0)
                    .to_string(),
            );

            // Forward supported headers
            for (forwarded_header, param_name) in [
                (HeaderType::Host, "HTTP_HOST"),
                (HeaderType::ContentType, "CONTENT_TYPE"),
                (HeaderType::Cookie, "HTTP_COOKIE"),
                (HeaderType::UserAgent, "HTTP_USER_AGENT"),
                (HeaderType::Authorization, "HTTP_AUTHORIZATION"),
            ] {
                if let Some(value) = request.headers.get(&forwarded_header) {
                    params.insert(param_name.into(), value.to_string());
                }
            }

            // Generate the FCGI request
            let empty_vec = Vec::new();
            let fcgi_request =
                FcgiRequest::new(params, request.content.as_ref().unwrap_or(&empty_vec), true);

            // Send the request to the PHP interpreter
            let stream_index = self.next_stream.fetch_add(1, Ordering::SeqCst) % self.streams.len();
            let mut stream = self.streams[stream_index].lock().unwrap();

            if let Err(e) = stream.write(&fcgi_request.encode()) {
                state
                    .logger
                    .error("PHP Plugin lost connection with the PHP server");
                state.logger.error(&format!("Error: {}", e));
                std::process::exit(0);
            }

            let mut records: Vec<FcgiRecord> = Vec::new();

            // Continually read responses until an `FcgiType::End` response is reached
            loop {
                match FcgiRecord::read_from(&*stream) {
                    Ok(record) => {
                        if record.fcgi_type != FcgiType::Stdout
                            && record.fcgi_type != FcgiType::Stderr
                        {
                            break;
                        }
                        records.push(record);
                    }
                    Err(e) => {
                        state
                            .logger
                            .error("PHP Plugin lost connection with the PHP server");
                        state.logger.error(&format!("Error: {}", e));
                        std::process::exit(0);
                    }
                }
            }

            // Assume the content to be UTF-8 and load it all
            let mut content_bytes: Vec<u8> = Vec::new();
            records.iter().fold(&mut content_bytes, |acc, val| {
                acc.extend(&val.content_data);
                acc
            });

            // Parse the headers
            let content = std::str::from_utf8(&content_bytes).unwrap();
            let mut headers = Headers::new();
            let mut content_split = content.splitn(2, "\r\n\r\n");
            let mut status = StatusCode::OK;

            for line in content_split.next().unwrap().lines() {
                let mut line_split = line.splitn(2, ':');
                let name = line_split.next().unwrap().trim();
                let value = line_split.next().map(|s| s.trim());

                if let Some(value) = value {
                    if name == "Status" {
                        if let Ok(new_status) = StatusCode::try_from(
                            value.split(' ').next().unwrap().parse::<u16>().unwrap(),
                        ) {
                            status = new_status;
                        }
                    } else {
                        headers.add(name, value);
                    }
                }
            }

            // Create a response
            let mut response =
                Response::empty(status).with_bytes(content_split.next().unwrap_or("").as_bytes());

            // Add the headers
            response.headers = headers;

            let status_code_number: u16 = status.into();
            let status_code_string: &str = status.into();

            state.logger.info(&format!(
                "{}: {} {} (PHP Plugin) {}",
                request.address, status_code_number, status_code_string, request.uri
            ));

            // Return the response
            Some(response)
        } else {
            // If the file requested was not found, allow Humphrey to handle the error

            None
        }
    }
}

// Declare the plugin
declare_plugin!(PhpPlugin, PhpPlugin::default);
