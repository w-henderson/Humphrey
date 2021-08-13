mod config;
mod server;

#[path = "tests/bin/mod.rs"]
mod bin_tests;

use config::ServerMode;
use server::*;

fn main() {
    match config::load_config(None) {
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
