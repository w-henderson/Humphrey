//! Provides a wrapper around the stream to allow for simpler APIs.

#[cfg(feature = "tls")]
use rustls::ServerConnection;

use std::io::{Error, Read, Write};
use std::marker::PhantomData;
use std::net::{SocketAddr, TcpStream};

/// Represents a connection to a remote client or server.
///
/// This is typically a wrapper around `TcpStream`, but is required to allow for a single API
///   to be used to process both regular and TLS connections.
pub enum Stream<'a> {
    /// A regular TCP stream.
    Tcp(TcpStream),
    /// A TLS stream.
    #[cfg(feature = "tls")]
    Tls(rustls::Stream<'a, ServerConnection, TcpStream>),
    /// Phantom data to contain the lifetime of the stream when the TLS feature is disabled.
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
    /// Returns the socket address of the remote peer of this connection.
    pub fn peer_addr(&self) -> Result<SocketAddr, Error> {
        match self {
            Stream::Tcp(stream) => stream.peer_addr(),
            #[cfg(feature = "tls")]
            Stream::Tls(stream) => stream.sock.peer_addr(),
            Stream::Phantom(_) => panic!("Phantom data in stream enum"),
        }
    }

    /// Shuts down both the read and write halves of this connection.
    pub fn shutdown(&self) -> std::io::Result<()> {
        match self {
            Stream::Tcp(stream) => stream.shutdown(std::net::Shutdown::Both),
            #[cfg(feature = "tls")]
            Stream::Tls(stream) => stream.sock.shutdown(std::net::Shutdown::Both),
            Stream::Phantom(_) => panic!("Phantom data in stream enum"),
        }
    }

    /// Sets this connection to nonblocking mode.
    pub fn set_nonblocking(&self) -> std::io::Result<()> {
        match self {
            Stream::Tcp(stream) => stream.set_nonblocking(true),
            #[cfg(feature = "tls")]
            Stream::Tls(stream) => stream.sock.set_nonblocking(true),
            Stream::Phantom(_) => panic!("Phantom data in stream enum"),
        }
    }

    /// Sets this connection to blocking mode.
    pub fn set_blocking(&self) -> std::io::Result<()> {
        match self {
            Stream::Tcp(stream) => stream.set_nonblocking(false),
            #[cfg(feature = "tls")]
            Stream::Tls(stream) => stream.sock.set_nonblocking(false),
            Stream::Phantom(_) => panic!("Phantom data in stream enum"),
        }
    }
}
