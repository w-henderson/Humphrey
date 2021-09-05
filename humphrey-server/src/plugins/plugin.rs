//! Types used when creating a plugin.
//!
//! https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html

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
    fn on_load(&mut self);
    /// Called when a request is received but before it is processed.
    fn on_request(&mut self, request: &mut Request, state: Arc<AppState>);
    /// Called when a response has been generated but not yet sent.
    fn on_response(&mut self, response: &mut Response, state: Arc<AppState>);
    /// Called when the plugin is about to be unloaded, should be used for any cleanup.
    fn on_unload(&mut self);
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
