//! Provides functionality for parsing and representing network addresses.

use crate::http::headers::{RequestHeader, RequestHeaderMap};

use std::error::Error;
use std::fmt::Display;
use std::net::{IpAddr, ToSocketAddrs};
use std::str::FromStr;

/// Represents a request's address.
/// Contains proxy data if applicable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Address {
    /// Original address of the request behind any proxies.
    pub origin_addr: IpAddr,
    /// The proxies that the request travelled through, from first to last.
    pub proxies: Vec<IpAddr>,
    /// The port of the request.
    pub port: u16,
}

impl Address {
    /// Create a new `Address` object from the socket address.
    pub fn new(addr: impl ToSocketAddrs) -> Result<Self, Box<dyn Error>> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or("No socket address found")?;

        Ok(Self {
            origin_addr: addr.ip(),
            proxies: Vec::new(),
            port: addr.port(),
        })
    }

    /// Create a new `Address` object from a request's headers and the socket address.
    /// This looks for the `X-Forwarded-For` header, used by proxies and CDNs, to find the origin address.
    pub fn from_headers(
        headers: &RequestHeaderMap,
        addr: impl ToSocketAddrs,
    ) -> Result<Self, Box<dyn Error>> {
        if let Some(forwarded) = headers.get(&RequestHeader::Custom {
            name: "x-forwarded-for".into(),
        }) {
            let mut proxies: Vec<IpAddr> = forwarded
                .split(',')
                .filter_map(|s| IpAddr::from_str(s).ok())
                .collect();

            if proxies.is_empty() {
                return Self::new(addr);
            }

            let origin_addr = *proxies.last().ok_or("No socket address found")?;
            proxies.remove(proxies.len() - 1);
            proxies.push(
                addr.to_socket_addrs()?
                    .next()
                    .ok_or("No socket address found")?
                    .ip(),
            );

            Ok(Self {
                origin_addr,
                proxies,
                port: addr
                    .to_socket_addrs()?
                    .next()
                    .ok_or("No socket address found")?
                    .port(),
            })
        } else {
            Self::new(addr)
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.proxies.is_empty() {
            write!(f, "{} (proxied)", self.origin_addr)
        } else {
            write!(f, "{}", self.origin_addr)
        }
    }
}
