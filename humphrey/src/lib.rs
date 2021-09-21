//! # Humphrey: A Performance-Focused, Dependency-Free Web Server.
//!
//! Humphrey is a very fast, robust and flexible HTTP/1.1 web server, with support for static and dynamic content through its Rust crate and plugin system. It has no dependencies when only using default features, and is easily extensible with a configuration file, dynamically-loaded plugins, or its Rust crate.

pub mod app;
pub mod http;
pub mod krauss;
pub mod route;
pub mod thread;

mod tests;

pub use app::App;
