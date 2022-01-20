//! Humphrey is a very fast, robust and flexible HTTP/1.1 web server, with support for static and dynamic content through its plugin system. It has no dependencies when only using default features, and is easily extensible with a configuration file and dynamically-loaded plugins.
//!
//! Learn more about Humphrey Server [here](https://humphrey.whenderson.dev/server/index.html).

#![warn(missing_docs)]

pub mod config;
pub mod server;
pub use server::*;

#[cfg(feature = "plugins")]
pub mod plugins;
