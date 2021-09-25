use humphrey::app::App;
use humphrey::http::headers::ResponseHeader;
use humphrey::http::proxy::proxy_request;
use humphrey::http::{Request, Response, StatusCode};

use crate::config::Config;
use crate::logger::Logger;

use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

/// Represents the application state.
/// Includes the proxy target and the logger.
struct AppState {
    target: SocketAddr,
    blacklist: Vec<String>,
    logger: Logger,
    timeout: Duration,
}

impl From<&Config> for AppState {
    fn from(config: &Config) -> Self {
        Self {
            target: config
                .proxy_target
                .as_ref()
                .unwrap()
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap(),
            blacklist: config.blacklist.clone(),
            logger: Logger::from(config),
            timeout: Duration::from_secs(5),
        }
    }
}

/// Main function for the proxy server.
pub fn main(config: Config) {
    let app: App<AppState> =
        App::new_with_config(config.threads, AppState::from(&config)).with_route("/*", handler);

    let addr = format!("{}:{}", config.address, config.port);

    let logger = &app.get_state().logger;
    logger.info("Parsed configuration, starting proxy server");
    logger.info(&format!(
        "Running at {}, proxying to {}",
        addr,
        config.proxy_target.as_ref().unwrap()
    ));
    logger.debug(&format!("Configuration: {:?}", &config));

    app.run(addr).unwrap();
}

/// Handles individual connections to the server.
fn handler(request: Request, state: Arc<AppState>) -> Response {
    // Return error 403 if the address was blacklisted
    if state
        .blacklist
        .contains(&request.address.origin_addr.to_string())
    {
        state.logger.warn(&format!(
            "{}: Blacklisted IP attempted to request {}",
            request.address, request.uri
        ));
        Response::new(StatusCode::Forbidden)
            .with_header(ResponseHeader::ContentType, "text/html".into())
            .with_bytes(b"<h1>403 Forbidden</h1>".to_vec())
            .with_request_compatibility(&request)
            .with_generated_headers()
    } else {
        let response = proxy_request(&request, state.target, state.timeout);
        let status: u16 = response.status_code.clone().into();
        let status_string: &str = response.status_code.clone().into();

        state.logger.info(&format!(
            "{}: {} {} {}",
            request.address, status, status_string, request.uri
        ));

        response
    }
}
