use humphrey::http::headers::ResponseHeaderMap;
use humphrey::http::{Request, Response, StatusCode};

use humphrey_server::config::extended_hashmap::ExtendedMap;
use humphrey_server::config::Config;
use humphrey_server::declare_plugin;
use humphrey_server::plugins::plugin::{Plugin, PluginLoadResult};
use humphrey_server::route::try_find_path;
use humphrey_server::static_server::AppState;

use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::net::{Shutdown, TcpStream};
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
        config: &Config,
        state: Arc<AppState>,
    ) -> PluginLoadResult<(), &'static str> {
        // Parses the configuration
        let php_address = config.raw.get_optional("php.address", "127.0.0.1".into());
        let php_port = config.raw.get_optional("php.port", "9000".into());
        let php_target = format!("{}:{}", php_address, php_port);
        let stream_count = config.raw.get_optional_parsed("php.threads", 8_usize, "");

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

    fn on_request(&self, request: &mut Request, state: Arc<AppState>) -> Option<Response> {
        if request.uri.split('.').last().unwrap() == "php" {
            // If the requested file is a PHP file, check that the file exists then process it
            if let Some(located) = try_find_path(&state.directory, &request.uri) {
                let file_name = located.path.to_str().unwrap().to_string();

                // On Windows, paths generated in this way have "\\?\" at the start
                // This means the path length limit is bypassed
                #[cfg(windows)]
                let file_name = file_name.replace("\\\\?\\", "");

                // Insert the parameters into a hashmap
                let mut params: HashMap<String, String> = HashMap::new();
                params.insert("GATEWAY_INTERFACE".into(), "FastCGI/1.0".into());
                params.insert("REQUEST_METHOD".into(), request.method.to_string());
                params.insert("SCRIPT_NAME".into(), request.uri.clone());
                params.insert("SCRIPT_FILENAME".into(), file_name);
                params.insert("QUERY_STRING".into(), request.query.clone());
                params.insert("SERVER_SOFTWARE".into(), "Humphrey".into());
                params.insert("REMOTE_ADDR".into(), "127.0.0.1".into());
                params.insert("SERVER_NAME".into(), "127.0.0.1".into());
                params.insert("SERVER_PORT".into(), "80".into());
                params.insert("SERVER_PROTOCOL".into(), "HTTP/1.1".into());
                params.insert(
                    "CONTENT_LENGTH".into(),
                    request
                        .content
                        .as_ref()
                        .map(|v| v.len())
                        .unwrap_or(0)
                        .to_string(),
                );

                // Generate the FCGI request
                let empty_vec = Vec::new();
                let fcgi_request =
                    FcgiRequest::new(params, request.content.as_ref().unwrap_or(&empty_vec), true);

                // Send the request to the PHP interpreter
                let stream_index =
                    self.next_stream.fetch_add(1, Ordering::SeqCst) % self.streams.len();
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
                            if record.fcgi_type != FcgiType::Stdout {
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

                // Assume the content to be UTF-8 and parse the headers
                let content = std::str::from_utf8(&records[0].content_data).unwrap();
                let mut headers: ResponseHeaderMap = BTreeMap::new();
                let mut content_split = content.splitn(2, "\r\n\r\n");

                for line in content_split.next().unwrap().lines() {
                    let mut line_split = line.splitn(2, ":");
                    let name = line_split.next().unwrap().trim();
                    let value = line_split.next().map(|s| s.trim());

                    if let Some(value) = value {
                        headers.insert(name.into(), value.into());
                    }
                }

                // Create a response
                let mut response = Response::new(StatusCode::OK)
                    .with_bytes(content_split.next().unwrap_or("").as_bytes().to_vec());

                // Add the headers
                response.headers = headers;

                state.logger.info(&format!(
                    "{}: 200 OK (PHP Plugin) {}",
                    request.address, request.uri
                ));

                // Add final headers and return the response
                Some(
                    response
                        .with_request_compatibility(&request)
                        .with_generated_headers(),
                )
            } else {
                // If the file requested was not found, allow Humphrey to handle the error

                None
            }
        } else {
            // If the file requested was not a PHP file, make no changes and allow Humphrey to handle it

            None
        }
    }

    fn on_unload(&mut self) {
        // Drain the streams iterator, shutting down every stream.
        for stream in self.streams.drain(..) {
            let stream = stream.lock().unwrap();
            stream.shutdown(Shutdown::Both).unwrap();
        }
    }
}

// Declare the plugin
declare_plugin!(PhpPlugin, PhpPlugin::default);
