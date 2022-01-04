#[cfg(feature = "tls")]
use rustls::ServerConnection;

use std::io::{Error, Read, Write};
use std::net::{SocketAddr, TcpStream};

#[cfg(not(feature = "tls"))]
pub struct Stream(TcpStream);

#[cfg(feature = "tls")]
pub struct Stream<'a>(rustls::Stream<'a, ServerConnection, TcpStream>);

#[cfg(not(feature = "tls"))]
impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

#[cfg(not(feature = "tls"))]
impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

#[cfg(feature = "tls")]
impl<'a> Read for Stream<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

#[cfg(feature = "tls")]
impl<'a> Write for Stream<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

#[cfg(not(feature = "tls"))]
impl Stream {
    pub fn new(stream: TcpStream) -> Self {
        Self(stream)
    }

    pub fn inner(&self) -> &TcpStream {
        &self.0
    }

    pub fn into_inner(self) -> TcpStream {
        self.0
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, Error> {
        self.0.peer_addr()
    }
}

#[cfg(feature = "tls")]
impl<'a> Stream<'a> {
    pub fn new(stream: rustls::Stream<'a, ServerConnection, TcpStream>) -> Self {
        Self(stream)
    }

    pub fn inner(&self) -> &rustls::Stream<'a, ServerConnection, TcpStream> {
        &self.0
    }

    pub fn into_inner(self) -> rustls::Stream<'a, ServerConnection, TcpStream> {
        self.0
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, Error> {
        self.0.sock.peer_addr()
    }
}
