//! # Humphrey: A Performance-Focused, Lightweight Web Server.

pub mod app;
pub mod http;
pub mod krauss;
pub mod route;

#[path = "tests/lib/mod.rs"]
mod lib_tests;

pub use app::App;
