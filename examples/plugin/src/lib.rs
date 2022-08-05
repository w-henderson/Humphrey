use humphrey::http::headers::HeaderType;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::stream::Stream;

use humphrey_server::config::RouteConfig;
use humphrey_server::declare_plugin;
use humphrey_server::plugins::plugin::Plugin;
use humphrey_server::server::server::AppState;

use humphrey_ws::{websocket_handler, Message, WebsocketStream};

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
                    .with_bytes(b"Response overridden by example plugin :)")
                    .with_header(HeaderType::ContentType, "text/plain"),
            );
        }

        None
    }

    fn on_websocket_request(
        &self,
        request: &mut Request,
        stream: Stream,
        state: Arc<AppState>,
        _: Option<&RouteConfig>,
    ) -> Option<Stream> {
        state.logger.info(&format!(
            "Example plugin read a WebSocket request from {}",
            request.address
        ));

        // If the requested resource is "/override" then override the response (which would ordinarily be closing the WebSocket connection). For this example, we'll just complete the WebSocket handshake and send a message back to the client.
        if &request.uri == "/override" {
            state
                .logger
                .info("Example plugin overrode a WebSocket connection");

            websocket_handler(|mut stream: WebsocketStream, _| {
                stream
                    .send(Message::new(b"Response overridden by example plugin :)"))
                    .ok();
            })(request.clone(), stream, state);

            return None;
        }

        Some(stream)
    }

    fn on_response(&self, response: &mut Response, state: Arc<AppState>) {
        // Insert a header to the response
        response.headers.add("X-Example-Plugin", "true");

        state
            .logger
            .info("Example plugin added the X-Example-Plugin header to a response");
    }
}

// Declare the plugin
declare_plugin!(ExamplePlugin, ExamplePlugin::default);
