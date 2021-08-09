use humphrey::app::{App, ErrorHandler};
use humphrey::route::RouteHandler;

use crate::config::{Config, LoadBalancerMode};
use crate::proxy::pipe;
use crate::server::rand::{Choose, Lcg};

use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

/// Main function for the load balancer.
pub fn main(config: Config) {
    let app: App<Mutex<LoadBalancer>> = App::new()
        .with_state(Mutex::new(LoadBalancer::from(&config)))
        .with_custom_connection_handler(handler);

    let addr = format!("{}:{}", config.address, config.port);

    app.run(addr).unwrap();
}

#[derive(Default)]
// Represents a load balancer.
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
    _: Arc<Vec<RouteHandler<Mutex<LoadBalancer>>>>,
    _: Arc<ErrorHandler>,
    load_balancer: Arc<Mutex<LoadBalancer>>,
) {
    let mut load_balancer_lock = load_balancer.lock().unwrap();
    let target = load_balancer_lock.select_target();
    drop(load_balancer_lock);

    if let Ok(mut destination) = TcpStream::connect(target) {
        let mut source_clone = source.try_clone().unwrap();
        let mut destination_clone = destination.try_clone().unwrap();

        let forward = spawn(move || pipe(&mut source, &mut destination));
        let backward = spawn(move || pipe(&mut destination_clone, &mut source_clone));

        forward.join().ok();
        backward.join().ok();
    }
}
