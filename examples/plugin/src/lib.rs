use humphrey::declare_plugin;
use humphrey::http::headers::ResponseHeader;
use humphrey::http::{Request, Response};
use humphrey::plugins::plugin::{Plugin, PluginLogger};

#[derive(Debug, Default)]
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &'static str {
        "Example Plugin"
    }

    fn on_load(&mut self, log: PluginLogger) {
        log("Example plugin loaded");
    }

    fn on_request(&mut self, request: &mut Request, log: PluginLogger) {
        log(&format!(
            "Example plugin read a request from {}",
            request.address
        ));
    }

    fn on_response(&mut self, response: &mut Response, log: PluginLogger) {
        // Insert a header to the response
        response.headers.insert(
            ResponseHeader::Custom {
                name: "X-Example-Plugin".into(),
            },
            "true".into(),
        );

        log("Example plugin added the X-Example-Plugin header to a response");
    }

    fn on_unload(&mut self, log: PluginLogger) {
        log("Example program was unloaded");
    }
}

// Declare the plugin
declare_plugin!(ExamplePlugin, ExamplePlugin::default);
