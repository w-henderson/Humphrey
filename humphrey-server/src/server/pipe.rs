use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};

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
