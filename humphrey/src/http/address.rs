use crate::http::headers::{RequestHeader, RequestHeaderMap};

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
    pub fn new(addr: impl ToSocketAddrs) -> Self {
        let addr = addr.to_socket_addrs().unwrap().next().unwrap();

        Self {
            origin_addr: addr.ip(),
            proxies: Vec::new(),
            port: addr.port(),
        }
    }

    /// Create a new `Address` object from a request's headers and the socket address.
    /// This looks for the `X-Forwarded-For` header, used by proxies and CDNs, to find the origin address.
    pub fn from_headers(headers: &RequestHeaderMap, addr: impl ToSocketAddrs) -> Self {
        if let Some(forwarded) = headers.get(&RequestHeader::Custom {
            name: "x-forwarded-for".into(),
        }) {
            let mut proxies: Vec<IpAddr> = forwarded
                .split(',')
                .map(|s| IpAddr::from_str(s).unwrap())
                .collect();

            let origin_addr = *proxies.last().unwrap();
            proxies.remove(proxies.len() - 1);
            proxies.push(addr.to_socket_addrs().unwrap().next().unwrap().ip());

            Self {
                origin_addr,
                proxies,
                port: addr.to_socket_addrs().unwrap().next().unwrap().port(),
            }
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
