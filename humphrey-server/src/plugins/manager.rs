//! Plugin management code.
//!
//! <https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html>

use crate::config::RouteConfig;
use crate::plugins::plugin::{Plugin, PluginLoadResult};
use crate::server::server::AppState;
use humphrey::http::{Request, Response};
use humphrey::stream::Stream;

use libloading::Library;
use std::collections::HashMap;
use std::sync::Arc;

/// Encapsulates plugins and their corresponding libraries.
#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    libraries: Vec<Library>,
}

impl PluginManager {
    /// Loads a plugin library.
    ///
    /// # Safety
    /// Calls foreign code.
    ///
    /// If the plugin that is being loaded is memory safe, then this function is memory safe.
    /// For example, if the plugin was written in Rust using the provided plugin API, it will be safe.
    pub unsafe fn load_plugin(
        &mut self,
        path: &str,
        config: &HashMap<String, String>,
        state: Arc<AppState>,
    ) -> PluginLoadResult<String, &'static str> {
        type PluginInitFunction = unsafe extern "C" fn() -> *mut dyn Plugin;

        // Load the plugin library, store it on the heap, and use a reference to the heap allocated instance
        // If the library doesn't load, return an error
        if let Ok(library) = Library::new(path) {
            self.libraries.push(library);
            let library = self.libraries.last().unwrap();

            // Get the initialisation function from the library
            // If the function can't be found, return an error
            if let Ok(init_function) = library.get::<PluginInitFunction>(b"_plugin_init") {
                // Load the plugin and store its instance on the heap
                let boxed_raw = init_function();
                let mut plugin = Box::from_raw(boxed_raw);

                // Run the plugin's load function
                let result = plugin.on_load(config, state);

                // If the result is ok, add the plugin to the list and return its name
                // Otherwise return the error message
                result.map(|_| {
                    let name = plugin.name().to_string();
                    self.plugins.push(plugin);
                    name
                })
            } else {
                PluginLoadResult::Fatal(
                    "Couldn't find plugin initialisation function in the library",
                )
            }
        } else {
            PluginLoadResult::Fatal("Couldn't load dynamic library")
        }
    }

    /// Calls the `on_request` function on every plugin.
    /// If a plugin overrides the response, this is immediately returned.
    pub fn on_request(
        &self,
        request: &mut Request,
        state: Arc<AppState>,
        route: &RouteConfig,
    ) -> Option<Response> {
        for plugin in &self.plugins {
            if let Some(response) = plugin.on_request(request, state.clone(), route) {
                return Some(response);
            }
        }

        None
    }

    /// Calls the `on_websocket_request` function on every plugin.
    /// If a plugin handles the stream, the function immediately returns.
    pub fn on_websocket_request(
        &self,
        request: &mut Request,
        mut stream: Stream,
        state: Arc<AppState>,
        route: Option<&RouteConfig>,
    ) -> Option<Stream> {
        for plugin in &self.plugins {
            if let Some(returned_stream) =
                plugin.on_websocket_request(request, stream, state.clone(), route)
            {
                stream = returned_stream;
            } else {
                return None;
            }
        }

        Some(stream)
    }

    /// Calls the `on_response` function on every plugin.
    pub fn on_response(&self, response: &mut Response, state: Arc<AppState>) {
        for plugin in &self.plugins {
            plugin.on_response(response, state.clone());
        }
    }

    /// Unloads every plugin.
    pub fn unload(&mut self) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_unload());

        for library in self.libraries.drain(..) {
            drop(library);
        }
    }

    /// Returns the number of currently loaded plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        if !self.plugins.is_empty() || !self.libraries.is_empty() {
            self.unload();
        }
    }
}
