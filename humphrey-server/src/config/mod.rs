//! Provides configuration functionality.

#![allow(clippy::module_inception)]

pub mod config;
pub mod default;
pub mod error;
pub mod extended_hashmap;
pub mod traceback;
pub mod tree;

pub use config::*;
