mod config;
mod server;
mod util;

use config::ServerMode;
use server::*;

fn main() {
    match config::load_config() {
        Ok(config) => match config.mode {
            ServerMode::Static => static_server::main(config),
            ServerMode::Proxy => proxy::main(config),
            ServerMode::LoadBalancer => load_balancer::main(config),
        },
        Err(error) => {
            println!("An error occurred loading the configuration: {}", error)
        }
    }
}
