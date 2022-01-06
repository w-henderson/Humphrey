//! Types used when creating a plugin.
//!
//! <https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html>

#![allow(unused_variables)]
#![allow(dead_code)]

use crate::config::RouteConfig;
use crate::server::server::AppState;
use humphrey::http::{Request, Response};

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// Represents a plugin.
pub trait Plugin: Any + Send + Sync + Debug {
    /// Returns the name of the plugin.
    fn name(&self) -> &'static str;

    /// Called when the plugin is first loaded.
    /// Any set-up that needs to be done before requests are handled should be done here.
    ///
    /// If the plugin cannot load for any reason, it should return `PluginLoadResult::NonFatal("error message")`
    ///   if the error is not fatal, for example configuration could not be loaded and defaults must be used, or
    ///   `PluginLoadResult::Fatal("error message")` if the error is fatal and will prevent the plugin from
    ///   working at all.
    fn on_load(
        &mut self,
        config: &HashMap<String, String>,
        state: Arc<AppState>,
    ) -> PluginLoadResult<(), &'static str> {
        PluginLoadResult::Ok(())
    }

    /// Called when a request is received but before it is processed. May modify the request in-place.
    /// Should return `None` to indicate that Humphrey should process the request,
    ///   or the plugin should process the request itself and return `Some(response)`.
    fn on_request(
        &self,
        request: &mut Request,
        state: Arc<AppState>,
        route: &RouteConfig,
    ) -> Option<Response> {
        None
    }

    /// Called when a response has been generated but not yet sent.
    /// May modify the response in-place.
    fn on_response(&self, response: &mut Response, state: Arc<AppState>) {}

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

/// Represents the result of loading a plugin.
/// It is based on the standard library's result, but with two error options for fatal and non-fatal errors.
#[must_use = "this `PluginLoadResult` may be a `NonFatal` or `Fatal` variant, which should be handled"]
pub enum PluginLoadResult<T, E> {
    /// A successful result
    Ok(T),
    /// A non-fatal error which can be worked around
    NonFatal(E),
    /// A fatal error
    Fatal(E),
}

impl<T, E> PluginLoadResult<T, E> {
    /// Returns `true` if the result is ok, and `false` if not.
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Returns `true` if the result is either of the two errors, and `false` if it is ok.
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if the result is a fatal error, and `false` if it is non-fatal or ok.
    pub const fn is_fatal(&self) -> bool {
        matches!(self, Self::Fatal(_))
    }

    /// Maps a `PluginLoadResult<T, E>` to `PluginLoadResult<U, E>` by applying a function to the contained `Ok` value.
    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> PluginLoadResult<U, E> {
        match self {
            Self::Ok(value) => PluginLoadResult::Ok(op(value)),
            Self::NonFatal(e) => PluginLoadResult::NonFatal(e),
            Self::Fatal(e) => PluginLoadResult::Fatal(e),
        }
    }
}

impl<T, E> PluginLoadResult<T, E>
where
    E: Debug,
{
    /// Returns the wrapped value, panics if the result is not ok.
    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(value) => value,
            Self::NonFatal(e) => panic!("Unwrap called on non-fatal error: {:?}", e),
            Self::Fatal(e) => panic!("Unwrap called on fatal error: {:?}", e),
        }
    }
}
