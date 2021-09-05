use humphrey::http::headers::ResponseHeader;
use humphrey::http::{Request, Response};

use humphrey_server::declare_plugin;
use humphrey_server::plugins::plugin::Plugin;
use humphrey_server::static_server::AppState;

use std::sync::Arc;

#[derive(Debug, Default)]
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &'static str {
        "Example Plugin"
    }

    fn on_load(&mut self) {}

    fn on_request(&mut self, request: &mut Request, state: Arc<AppState>) {
        state.logger.info(&format!(
            "Example plugin read a request from {}",
            request.address
        ));
    }

    fn on_response(&mut self, response: &mut Response, state: Arc<AppState>) {
        // Insert a header to the response
        response.headers.insert(
            ResponseHeader::Custom {
                name: "X-Example-Plugin".into(),
            },
            "true".into(),
        );

        state
            .logger
            .info("Example plugin added the X-Example-Plugin header to a response");
    }

    fn on_unload(&mut self) {}
}

// Declare the plugin
declare_plugin!(ExamplePlugin, ExamplePlugin::default);
