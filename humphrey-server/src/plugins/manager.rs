//! Plugin management code.
//!
//! https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html

use crate::plugins::plugin::Plugin;
use crate::static_server::AppState;
use humphrey::http::{Request, Response};

use libloading::{Library, Symbol};
use std::sync::Arc;

/// Encapsulates plugins and their corresponding libraries.
#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    libraries: Vec<Library>,
}

impl PluginManager {
    /// Loads a plugin library.
    pub unsafe fn load_plugin(&mut self, path: &str) -> Result<String, &str> {
        type PluginInitFunction = unsafe extern "C" fn() -> *mut dyn Plugin;

        // Load the plugin library, store it on the heap, and use a reference to the heap allocated instance
        let library = Library::new(path).map_err(|_| "Couldn't load library")?;
        self.libraries.push(library);
        let library = self.libraries.last().unwrap();

        // Get the initialisation function from the library
        let init_function: Symbol<PluginInitFunction> = library
            .get(b"_plugin_init")
            .map_err(|_| "Initialisation function not found")?;

        // Load the plugin and store its instance on the heap
        let boxed_raw = init_function();
        let mut plugin = Box::from_raw(boxed_raw);
        plugin.on_load();

        let name = plugin.name().to_string();

        self.plugins.push(plugin);

        Ok(name)
    }

    /// Calls the `on_request` function on every plugin.
    /// If a plugin overrides the response, this is immediately returned.
    pub fn on_request(&mut self, request: &mut Request, state: Arc<AppState>) -> Option<Response> {
        for plugin in &mut self.plugins {
            if let Some(response) = plugin.on_request(request, state.clone()) {
                return Some(response);
            }
        }

        None
    }

    /// Calls the `on_response` function on every plugin.
    pub fn on_response(&mut self, response: &mut Response, state: Arc<AppState>) {
        for plugin in &mut self.plugins {
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
