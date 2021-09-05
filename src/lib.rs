//! # Humphrey: A Performance-Focused, Lightweight Web Server.

pub mod app;
pub mod http;
pub mod krauss;
pub mod route;

#[cfg(feature = "plugins")]
#[path = "plugins/mod.rs"]
pub mod plugins;

#[path = "tests/lib/mod.rs"]
mod lib_tests;

pub use app::App;
