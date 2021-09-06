use humphrey::http::headers::ResponseHeaderMap;
use humphrey::http::{Request, Response, StatusCode};

use humphrey_server::declare_plugin;
use humphrey_server::plugins::plugin::Plugin;
use humphrey_server::route::try_find_path;
use humphrey_server::static_server::AppState;

use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::fcgi::record::FcgiRecord;
use crate::fcgi::request::FcgiRequest;
use crate::fcgi::types::FcgiType;

pub mod fcgi;

#[derive(Debug, Default)]
pub struct PhpPlugin {
    cgi_stream: Option<Mutex<TcpStream>>,
}

impl Plugin for PhpPlugin {
    fn name(&self) -> &'static str {
        "PHP Plugin"
    }

    fn on_load(&mut self) -> Result<(), &'static str> {
        let stream = TcpStream::connect("127.0.0.1:9000")
            .map_err(|_| "Could not connect to the PHP CGI server on port 9000")?;
        self.cgi_stream = Some(Mutex::new(stream));

        Ok(())
    }

    fn on_request(&mut self, request: &mut Request, state: Arc<AppState>) -> Option<Response> {
        if request.uri.split('.').last().unwrap() == "php" {
            let full_path = format!("{}{}", state.directory, request.uri);

            if let Some(located) = try_find_path(&full_path) {
                let stream_mutex = self.cgi_stream.as_ref().unwrap();
                let mut stream = stream_mutex.lock().unwrap();

                let file_name = located.path.to_str().unwrap().to_string();

                #[cfg(windows)]
                let file_name = file_name.replace("\\\\?\\", "");

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

                let empty_vec = Vec::new();
                let fcgi_request =
                    FcgiRequest::new(params, request.content.as_ref().unwrap_or(&empty_vec), true);

                stream.write(&fcgi_request.encode()).unwrap();

                let mut records: Vec<FcgiRecord> = Vec::new();

                loop {
                    let record = FcgiRecord::from(&mut stream);
                    if record.fcgi_type == FcgiType::End {
                        break;
                    }
                    records.push(record);
                }

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

                let mut response = Response::new(StatusCode::OK)
                    .with_bytes(content_split.next().unwrap_or("").as_bytes().to_vec());

                response.headers = headers;

                state.logger.info(&format!(
                    "{}: 200 OK (PHP Plugin) {}",
                    request.address, request.uri
                ));

                Some(
                    response
                        .with_request_compatibility(&request)
                        .with_generated_headers(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }
}

// Declare the plugin
declare_plugin!(PhpPlugin, PhpPlugin::default);
