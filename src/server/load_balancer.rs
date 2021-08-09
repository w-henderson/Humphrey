use humphrey::app::{App, ErrorHandler};
use humphrey::route::RouteHandler;

use crate::config::{Config, LoadBalancerMode};
use crate::logger::Logger;
use crate::proxy::pipe;
use crate::server::rand::{Choose, Lcg};

use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

#[derive(Default)]
struct AppState {
    load_balancer: Mutex<LoadBalancer>,
    logger: Logger,
}

impl From<&Config> for AppState {
    fn from(config: &Config) -> Self {
        Self {
            load_balancer: Mutex::new(LoadBalancer::from(config)),
            logger: Logger::from(config),
        }
    }
}

/// Main function for the load balancer.
pub fn main(config: Config) {
    let app: App<AppState> = App::new()
        .with_state(AppState::from(&config))
        .with_custom_connection_handler(handler);

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
fn handler(
    mut source: TcpStream,
    _: Arc<Vec<RouteHandler<AppState>>>,
    _: Arc<ErrorHandler>,
    state: Arc<AppState>,
) {
    let mut load_balancer_lock = state.load_balancer.lock().unwrap();
    let target = load_balancer_lock.select_target();
    drop(load_balancer_lock);

    if let Ok(mut destination) = TcpStream::connect(&target) {
        let address = source.peer_addr().unwrap().to_string();
        state
            .logger
            .info(&format!("{} -> {}: Connection started", &address, &target));

        let mut source_clone = source.try_clone().unwrap();
        let mut destination_clone = destination.try_clone().unwrap();

        let forward = spawn(move || pipe(&mut source, &mut destination));
        let backward = spawn(move || pipe(&mut destination_clone, &mut source_clone));

        if let Err(_) = forward.join().unwrap() {
            state.logger.error(&format!(
                "{}: Error proxying data from client to target, connection closed",
                &address
            ));
        }
        if let Err(_) = backward.join().unwrap() {
            state.logger.error(&format!(
                "{}: Error proxying data from target to client, connection closed",
                &address
            ));
        }

        state.logger.info(&format!(
            "{} -> {}: Session complete, connection closed",
            &address, &target
        ));
    } else {
        state.logger.error(&format!(
            "Could not connect to load balancer target {}",
            target
        ))
    }
}
