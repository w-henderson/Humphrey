//! Provides the server functionality.

#![allow(clippy::module_inception)]

pub mod cache;
pub mod logger;
pub mod proxy;
pub mod rand;
pub mod server;
pub mod r#static;

pub use server::*;
