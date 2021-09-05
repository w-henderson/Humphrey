//! Plugin management code.
//!
//! https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html

use crate::http::{Request, Response};
use crate::plugins::plugin::{Plugin, PluginLogger};

use libloading::{Library, Symbol};

#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    libraries: Vec<Library>,
}

impl PluginManager {
    /// Loads a plugin library.
    pub unsafe fn load_plugin(&mut self, path: &str, logger: PluginLogger) -> Result<(), &str> {
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
        println!("{}", plugin.name());
        println!("{:?}", plugin);
        plugin.on_load(logger);

        self.plugins.push(plugin);

        Ok(())
    }

    /// Calls the `on_request` function on every plugin.
    pub fn on_request(&mut self, request: &mut Request, logger: PluginLogger) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_request(request, logger));
    }

    /// Calls the `on_response` function on every plugin.
    pub fn on_response(&mut self, response: &mut Response, logger: PluginLogger) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_response(response, logger));
    }

    /// Unloads every plugin.
    pub fn unload(&mut self, logger: PluginLogger) {
        self.plugins
            .iter_mut()
            .for_each(|plugin| plugin.on_unload(logger));

        for library in self.libraries.drain(..) {
            drop(library);
        }
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        if !self.plugins.is_empty() || !self.libraries.is_empty() {
            self.unload(|_| ());
        }
    }
}
