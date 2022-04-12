//! Provides HTTP proxy functionality.

use crate::config::LoadBalancerMode;
use crate::rand::{Choose, Lcg};
use crate::server::server::AppState;

use humphrey::http::headers::HeaderType;
use humphrey::http::proxy::proxy_request;
use humphrey::http::{Request, Response, StatusCode};

use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// Represents a load balancer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadBalancer {
    /// The targets of the load balancer.
    pub targets: Vec<String>,
    /// The algorithm used to choose a target.
    pub mode: LoadBalancerMode,
    /// The current target.
    pub index: usize,
    /// The random number generator used by the load balancer.
    pub lcg: Lcg,
}

impl LoadBalancer {
    /// Selects a target according to the load balancer mode.
    pub fn select_target(&mut self) -> String {
        match self.mode {
            LoadBalancerMode::RoundRobin => {
                let target_index = self.index;
                self.index += 1;
                if self.index == self.targets.len() {
                    self.index = 0;
                }

                self.targets[target_index].clone()
            }
            LoadBalancerMode::Random => self.targets.choose(&mut self.lcg).unwrap().clone(),
        }
    }
}

/// Handles proxy requests.
pub fn proxy_handler(
    request: Request,
    state: Arc<AppState>,
    load_balancer: &EqMutex<LoadBalancer>,
    matches: &str,
) -> Response {
    let mut simplified_uri = request.uri.clone();

    for ch in matches.chars() {
        if ch != '*' {
            simplified_uri.remove(0);
        } else {
            break;
        }
    }

    if !simplified_uri.starts_with('/') {
        simplified_uri.insert(0, '/');
    }

    // Return error 403 if the address was blacklisted
    if state
        .config
        .blacklist
        .list
        .contains(&request.address.origin_addr)
    {
        state.logger.warn(&format!(
            "{}: Blacklisted IP attempted to request {}",
            request.address, request.uri
        ));
        Response::empty(StatusCode::Forbidden)
            .with_header(HeaderType::ContentType, "text/html")
            .with_bytes(b"<h1>403 Forbidden</h1>")
    } else {
        // Gets a load balancer target using the thread-safe `Mutex`
        let mut load_balancer_lock = load_balancer.lock().unwrap();
        let target = load_balancer_lock.select_target();
        drop(load_balancer_lock);

        let mut proxied_request = request.clone();
        proxied_request.uri = simplified_uri;

        let target_sock = target.to_socket_addrs().unwrap().next().unwrap();
        let response = proxy_request(&proxied_request, target_sock, Duration::from_secs(5));
        let status: u16 = response.status_code.into();
        let status_string: &str = response.status_code.into();

        state.logger.info(&format!(
            "{}: {} {} {}",
            request.address, status, status_string, request.uri
        ));

        response
    }
}

/// A `Mutex` which implements `PartialEq` for testing.
#[derive(Debug)]
pub struct EqMutex<T> {
    mutex: Mutex<T>,
}

impl<T> EqMutex<T> {
    /// Locks the mutex.
    pub fn lock(&self) -> Result<MutexGuard<T>, PoisonError<MutexGuard<T>>> {
        self.mutex.lock()
    }

    /// Creates a new mutex.
    pub fn new(data: T) -> Self {
        Self {
            mutex: Mutex::new(data),
        }
    }
}

impl<T> PartialEq for EqMutex<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        *self.lock().unwrap() == *other.lock().unwrap()
    }
}
