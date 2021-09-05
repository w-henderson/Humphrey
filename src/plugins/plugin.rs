//! Types used when creating a plugin.
//!
//! https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html

use crate::http::{Request, Response};

use std::any::Any;
use std::fmt::Debug;

pub type PluginLogger = fn(&str);

pub trait Plugin: Any + Send + Sync + Debug {
    fn name(&self) -> &'static str;

    fn on_load(&mut self, log: PluginLogger);
    fn on_request(&mut self, request: &mut Request, log: PluginLogger);
    fn on_response(&mut self, response: &mut Response, log: PluginLogger);
    fn on_unload(&mut self, log: PluginLogger);
}

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
