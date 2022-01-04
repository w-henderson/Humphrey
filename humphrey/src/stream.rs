#[cfg(feature = "tls")]
use rustls::ServerConnection;

use std::io::{Error, Read, Write};
use std::marker::PhantomData;
use std::net::{SocketAddr, TcpStream};

pub enum Stream<'a> {
    Tcp(TcpStream),
    #[cfg(feature = "tls")]
    Tls(rustls::Stream<'a, ServerConnection, TcpStream>),
    Phantom(PhantomData<&'a ()>),
}

impl<'a> Read for Stream<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Stream::Tcp(stream) => stream.read(buf),
            #[cfg(feature = "tls")]
            Stream::Tls(stream) => stream.read(buf),
            Stream::Phantom(_) => panic!("Phantom data in stream enum"),
        }
    }
}

impl<'a> Write for Stream<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Stream::Tcp(stream) => stream.write(buf),
            #[cfg(feature = "tls")]
            Stream::Tls(stream) => stream.write(buf),
            Stream::Phantom(_) => panic!("Phantom data in stream enum"),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Stream::Tcp(stream) => stream.flush(),
            #[cfg(feature = "tls")]
            Stream::Tls(stream) => stream.flush(),
            Stream::Phantom(_) => panic!("Phantom data in stream enum"),
        }
    }
}

impl<'a> Stream<'a> {
    pub fn peer_addr(&self) -> Result<SocketAddr, Error> {
        match self {
            Stream::Tcp(stream) => stream.peer_addr(),
            #[cfg(feature = "tls")]
            Stream::Tls(stream) => stream.sock.peer_addr(),
            Stream::Phantom(_) => panic!("Phantom data in stream enum"),
        }
    }
}
