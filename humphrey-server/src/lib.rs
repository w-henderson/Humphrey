//! # Humphrey Server
//! Humphrey is a very fast, robust and flexible HTTP/1.1 web server, with support for static and dynamic content through its plugin system. It has no dependencies when only using default features, and is easily extensible with a configuration file and dynamically-loaded plugins.
//!
//! Read more [here](https://github.com/w-henderson/Humphrey/blob/master/humphrey-server/README.md).

#![warn(missing_docs)]

pub mod config;
pub mod server;
pub use server::*;

#[cfg(feature = "plugins")]
pub mod plugins;
