//! Provides a wrapper around the stream to allow for simpler APIs.

#![allow(clippy::large_enum_variant)]

#[cfg(feature = "tls")]
use tokio_rustls::server::TlsStream;

use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;

/// Represents a connection to a remote client or server.
///
/// This is typically a wrapper around `TcpStream`, but is required to allow for a single API
///   to be used to process both regular and TLS connections.
pub enum Stream {
    /// A regular TCP stream.
    Tcp(TcpStream),
    /// A TLS stream.
    #[cfg(feature = "tls")]
    Tls(TlsStream<TcpStream>),
}

impl AsyncRead for Stream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match &mut *self {
            Stream::Tcp(inner) => Pin::new(inner).poll_read(cx, buf),
            #[cfg(feature = "tls")]
            Stream::Tls(inner) => Pin::new(inner).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match &mut *self {
            Stream::Tcp(inner) => Pin::new(inner).poll_write(cx, buf),
            #[cfg(feature = "tls")]
            Stream::Tls(inner) => Pin::new(inner).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match &mut *self {
            Stream::Tcp(inner) => Pin::new(inner).poll_flush(cx),
            #[cfg(feature = "tls")]
            Stream::Tls(inner) => Pin::new(inner).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match &mut *self {
            Stream::Tcp(inner) => Pin::new(inner).poll_shutdown(cx),
            #[cfg(feature = "tls")]
            Stream::Tls(inner) => Pin::new(inner).poll_shutdown(cx),
        }
    }
}

impl Stream {
    /// Returns the socket address of the remote peer of this connection.
    pub fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        match self {
            Stream::Tcp(stream) => stream.peer_addr(),
            #[cfg(feature = "tls")]
            Stream::Tls(stream) => stream.get_ref().0.peer_addr(),
        }
    }

    /// Shuts down both the read and write halves of this connection.
    pub async fn shutdown(&mut self) -> std::io::Result<()> {
        match self {
            Stream::Tcp(stream) => stream.shutdown().await,
            #[cfg(feature = "tls")]
            Stream::Tls(stream) => stream.get_mut().0.shutdown().await,
        }
    }
}
