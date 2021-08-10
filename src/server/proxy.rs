use humphrey::app::{App, ErrorHandler};
use humphrey::route::RouteHandler;

use crate::config::Config;
use crate::logger::Logger;

use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::thread::spawn;

/// Represents the application state.
/// Includes the proxy target and the logger.
#[derive(Default)]
struct AppState {
    target: String,
    logger: Logger,
}

impl From<&Config> for AppState {
    fn from(config: &Config) -> Self {
        Self {
            target: config.proxy_target.as_ref().unwrap().clone(),
            logger: Logger::from(config),
        }
    }
}

/// Main function for the proxy server.
pub fn main(config: Config) {
    let app: App<AppState> = App::new()
        .with_state(AppState::from(&config))
        .with_custom_connection_handler(handler);

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
/// Ignores the server's specified routes and error handler.
fn handler(
    mut source: TcpStream,
    _: Arc<Vec<RouteHandler<AppState>>>,
    _: Arc<ErrorHandler>,
    state: Arc<AppState>,
) {
    let address = source.peer_addr().unwrap().to_string();

    if let Ok(mut destination) = TcpStream::connect(&*state.target) {
        // The target was successfully connected to
        let mut source_clone = source.try_clone().unwrap();
        let mut destination_clone = destination.try_clone().unwrap();
        state
            .logger
            .info(&format!("{}: Connected, proxying data", &address));

        // Pipe data in both directions
        let forward = spawn(move || pipe(&mut source, &mut destination));
        let backward = spawn(move || pipe(&mut destination_clone, &mut source_clone));

        // Log any errors
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
            "{}: Session complete, connection closed",
            &address
        ));
    } else {
        state.logger.error("Could not connect to target");
    }
}

/// Pipe bytes from one stream to another, up to 1KiB at a time.
pub fn pipe(source: &mut TcpStream, destination: &mut TcpStream) -> Result<(), ()> {
    let mut buf: [u8; 1024] = [0; 1024];

    loop {
        let length = source.read(&mut buf).map_err(|_| ())?;

        if length == 0 {
            destination.shutdown(Shutdown::Both).map_err(|_| ())?;
            break;
        }

        if let Ok(_) = destination.write(&buf[..length]) {
            destination.flush().map_err(|_| ())?;
        }
    }
    Ok(())
}
