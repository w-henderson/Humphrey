use crate::config::LoadBalancerMode;
use crate::rand::{Choose, Lcg};
use crate::server::server::AppState;

use humphrey::http::headers::ResponseHeader;
use humphrey::http::proxy::proxy_request;
use humphrey::http::{Request, Response, StatusCode};

use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// Represents a load balancer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadBalancer {
    pub targets: Vec<String>,
    pub mode: LoadBalancerMode,
    pub index: usize,
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

pub fn proxy_handler(
    request: Request,
    state: Arc<AppState>,
    load_balancer: &EqMutex<LoadBalancer>,
) -> Response {
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
        Response::new(StatusCode::Forbidden)
            .with_header(ResponseHeader::ContentType, "text/html".into())
            .with_bytes(b"<h1>403 Forbidden</h1>".to_vec())
            .with_request_compatibility(&request)
            .with_generated_headers()
    } else {
        // Gets a load balancer target using the thread-safe `Mutex`
        let mut load_balancer_lock = load_balancer.lock().unwrap();
        let target = load_balancer_lock.select_target();
        drop(load_balancer_lock);

        let target_sock = target.to_socket_addrs().unwrap().next().unwrap();
        let response = proxy_request(&request, target_sock, Duration::from_secs(5));
        let status: u16 = response.status_code.clone().into();
        let status_string: &str = response.status_code.clone().into();

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
    pub fn lock(&self) -> Result<MutexGuard<T>, PoisonError<MutexGuard<T>>> {
        self.mutex.lock()
    }

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
