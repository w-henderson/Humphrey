//! Humphrey is a very fast, robust and flexible HTTP/1.1 web server crate which allows you to develop web applications in Rust. With no dependencies, it is very quick to compile and produces very small binaries, as well as being very resource-efficient.
//!
//! Learn more about Humphrey [here](https://humphrey.whenderson.dev/core/index.html).

#![warn(missing_docs)]

pub mod app;
pub mod handlers;
pub mod http;
pub mod krauss;
pub mod monitor;
pub mod percent;
pub mod route;
pub mod stream;
pub mod thread;

#[cfg(test)]
mod tests;

pub use app::App;
pub use route::SubApp;
