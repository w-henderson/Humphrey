//! Types used when creating a plugin.
//!
//! https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html

use crate::http::{Request, Response};

use std::any::Any;
use std::fmt::Debug;

/// Represents a logging function for the plugin.
pub type PluginLogger = fn(&str);

/// Represents a plugin.
pub trait Plugin: Any + Send + Sync + Debug {
    /// Returns the name of the plugin.
    fn name(&self) -> &'static str;

    /// Called when the plugin is first loaded.
    fn on_load(&mut self, log: PluginLogger);
    /// Called when a request is received but before it is processed.
    fn on_request(&mut self, request: &mut Request, log: PluginLogger);
    /// Called when a response has been generated but not yet sent.
    fn on_response(&mut self, response: &mut Response, log: PluginLogger);
    /// Called when the plugin is about to be unloaded, should be used for any cleanup.
    fn on_unload(&mut self, log: PluginLogger);
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
