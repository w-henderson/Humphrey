//! Humphrey is a very fast, robust and flexible HTTP/1.1 web server crate which allows you to develop web applications in Rust. With no dependencies, it is very quick to compile and produces very small binaries, as well as being very resource-efficient.
//!
//! Learn more about Humphrey [here](https://humphrey.whenderson.dev/core/index.html).

#![warn(missing_docs)]

#[cfg(not(feature = "tokio"))]
pub mod app;
#[cfg(not(feature = "tokio"))]
pub mod handler_traits;
#[cfg(not(feature = "tokio"))]
pub mod handlers;

#[cfg(feature = "tokio")]
#[allow(missing_docs)]
pub mod tokio;
#[cfg(feature = "tokio")]
pub use crate::tokio::*;

#[cfg(not(feature = "tokio"))]
pub mod stream;

pub mod client;
pub mod http;
pub mod krauss;
pub mod monitor;
pub mod percent;
pub mod route;
pub mod thread;

#[cfg(test)]
mod tests;

pub use app::App;
pub use client::Client;
pub use route::SubApp;
