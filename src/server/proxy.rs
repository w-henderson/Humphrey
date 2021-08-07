use humphrey::app::{App, ErrorHandler};
use humphrey::route::RouteHandler;

use crate::config::Config;

use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::thread::spawn;

/// Main function for the proxy server.
pub fn main(config: Config) {
    let app: App<String> = App::new()
        .with_state(config.proxy_target.unwrap())
        .with_custom_connection_handler(handler);

    let addr = format!("{}:{}", config.address, config.port);

    app.run(addr).unwrap();
}

/// Handles individual connections to the server.
/// Ignores the server's specified routes and error handler.
fn handler(
    mut source: TcpStream,
    _: Arc<Vec<RouteHandler<String>>>,
    _: Arc<ErrorHandler>,
    target: Arc<String>,
) {
    let mut destination = TcpStream::connect(&*target).unwrap();
    let mut source_clone = source.try_clone().unwrap();
    let mut destination_clone = destination.try_clone().unwrap();

    let forward = spawn(move || pipe(&mut source, &mut destination));
    let backward = spawn(move || pipe(&mut destination_clone, &mut source_clone));

    forward.join().unwrap().unwrap();
    backward.join().unwrap().unwrap();
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
