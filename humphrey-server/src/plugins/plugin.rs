//! Types used when creating a plugin.
//!
//! https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html

#![allow(unused_variables)]

use crate::static_server::AppState;
use humphrey::http::{Request, Response};

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

/// Represents a plugin.
pub trait Plugin: Any + Send + Sync + Debug {
    /// Returns the name of the plugin.
    fn name(&self) -> &'static str;

    /// Called when the plugin is first loaded.
    /// Any set-up that needs to be done before requests are handled should be done here.
    /// If the plugin cannot load for any reason, it should return `Err("error message")`.
    fn on_load(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    /// Called when a request is received but before it is processed. May modify the request in-place.
    /// Should return `None` to indicate that Humphrey should process the request,
    ///   or the plugin should process the request itself and return `Some(response)`.
    fn on_request(&mut self, request: &mut Request, state: Arc<AppState>) -> Option<Response> {
        None
    }

    /// Called when a response has been generated but not yet sent.
    /// May modify the response in-place.
    fn on_response(&mut self, response: &mut Response, state: Arc<AppState>) {}

    /// Called when the plugin is about to be unloaded.
    /// Any clean-up should be done here.
    fn on_unload(&mut self) {}
}

/// Declares the required functions for initialising a plugin.
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_init() -> *mut dyn Plugin {
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<dyn Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}
