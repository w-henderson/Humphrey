use humphrey::app::App;
use humphrey::http::headers::ResponseHeader;
use humphrey::http::proxy::proxy_request;
use humphrey::http::{Request, Response, StatusCode};

use crate::config::{Config, LoadBalancerMode};
use crate::logger::Logger;
use crate::server::rand::{Choose, Lcg};

use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Represents the load balancer application state.
/// Includes the load balancer instance as well as the logger.
#[derive(Default)]
struct AppState {
    load_balancer: Mutex<LoadBalancer>,
    blacklist: Vec<String>,
    logger: Logger,
    timeout: Duration,
}

impl From<&Config> for AppState {
    fn from(config: &Config) -> Self {
        Self {
            load_balancer: Mutex::new(LoadBalancer::from(config)),
            blacklist: config.blacklist.clone(),
            logger: Logger::from(config),
            timeout: Duration::from_secs(5),
        }
    }
}

/// Main function for the load balancer.
pub fn main(config: Config) {
    let app: App<AppState> =
        App::new_with_config(config.threads, AppState::from(&config)).with_route("/*", handler);

    let addr = format!("{}:{}", config.address, config.port);

    let logger = &app.get_state().logger;
    logger.info("Parsed configuration, starting load balancer");
    logger.info(&format!("Running at {}", addr));
    logger.info(&format!(
        "Using load balancer mode {:?}",
        &config.load_balancer_mode.as_ref().unwrap()
    ));
    logger.debug(&format!("Configuration: {:?}", &config));

    app.run(addr).unwrap();
}

// Represents a load balancer.
#[derive(Default)]
struct LoadBalancer {
    targets: Vec<String>,
    mode: LoadBalancerMode,
    index: usize,
    lcg: Lcg,
}

impl LoadBalancer {
    /// Selects a target according to the load balancer mode.
    pub fn select_target(&mut self) -> String {
        match self.mode {
            LoadBalancerMode::RoundRobin => {
                let target_index = self.index;
                self.index += 1;
                if self.index == self.targets.len() {
                    self.index = 0;
                }

                self.targets[target_index].clone()
            }
            LoadBalancerMode::Random => self.targets.choose(&mut self.lcg).unwrap().clone(),
        }
    }
}

impl From<&Config> for LoadBalancer {
    fn from(config: &Config) -> Self {
        Self {
            targets: config.load_balancer_targets.clone().unwrap(),
            mode: config.load_balancer_mode.clone().unwrap(),
            index: 0,
            lcg: Lcg::new(),
        }
    }
}

/// Handles individual connections to the server.
/// Ignores the server's specified routes and error handler.
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
        // Gets a load balancer target using the thread-safe `Mutex`
        let mut load_balancer_lock = state.load_balancer.lock().unwrap();
        let target = load_balancer_lock.select_target();
        drop(load_balancer_lock);

        let target_sock = target.to_socket_addrs().unwrap().next().unwrap();
        let response = proxy_request(&request, target_sock, state.timeout);
        let status: u16 = response.status_code.clone().into();
        let status_string: &str = response.status_code.clone().into();

        state.logger.info(&format!(
            "{}: {} {} {}",
            request.address, status, status_string, request.uri
        ));

        response
    }
}
