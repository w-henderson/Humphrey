use humphrey::http::headers::ResponseHeader;
use humphrey::http::{Request, Response, StatusCode};

use humphrey_server::config::RouteConfig;
use humphrey_server::declare_plugin;
use humphrey_server::plugins::plugin::Plugin;
use humphrey_server::server::server::AppState;

use std::sync::Arc;

#[derive(Debug, Default)]
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &'static str {
        "Example Plugin"
    }

    fn on_request(
        &self,
        request: &mut Request,
        state: Arc<AppState>,
        _: &RouteConfig,
    ) -> Option<Response> {
        state.logger.info(&format!(
            "Example plugin read a request from {}",
            request.address
        ));

        // If the requested resource is "/override" then override the response (which would ordinarily be 404).
        if &request.uri == "/override" {
            state.logger.info("Example plugin overrode a response");

            return Some(
                Response::empty(StatusCode::OK)
                    .with_bytes(b"Response overridden by example plugin :)".to_vec())
                    .with_header(ResponseHeader::ContentType, "text/plain".into()),
            );
        }

        None
    }

    fn on_response(&self, response: &mut Response, state: Arc<AppState>) {
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
}

// Declare the plugin
declare_plugin!(ExamplePlugin, ExamplePlugin::default);
