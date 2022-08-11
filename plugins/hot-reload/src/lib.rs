mod injector;
mod listen;

use humphrey::http::{Request, Response, StatusCode};

use humphrey::stream::Stream;
use humphrey_server::config::extended_hashmap::ExtendedMap;
use humphrey_server::config::RouteConfig;
use humphrey_server::declare_plugin;
use humphrey_server::plugins::plugin::{Plugin, PluginLoadResult};
use humphrey_server::AppState;

use humphrey_ws::{websocket_handler, WebsocketStream};

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

static INJECTED_JS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/inject.js"));

#[derive(Default)]
pub struct HotReloadPlugin {
    ws_route: String,
    streams: Arc<Mutex<Vec<WebsocketStream>>>,
}

impl Plugin for HotReloadPlugin {
    fn name(&self) -> &'static str {
        "Hot Reload"
    }

    fn on_load(
        &mut self,
        config: &HashMap<String, String>,
        state: Arc<AppState>,
    ) -> PluginLoadResult<(), &'static str> {
        if !state.config.hosts.is_empty() {
            return PluginLoadResult::NonFatal(
                "Warning: Hot Reload plugin cannot be used with custom host configuration",
            );
        }

        self.ws_route = config.get_optional("ws_route", "/__hot-reload-ws".into());

        if listen::init(self.streams.clone(), state).is_err() {
            return PluginLoadResult::Fatal(
                "Could not initialise Hot Reload plugin due to listener error",
            );
        }

        PluginLoadResult::Ok(())
    }

    fn on_websocket_request(
        &self,
        request: &mut Request,
        stream: Stream,
        state: Arc<AppState>,
        _: Option<&RouteConfig>,
    ) -> Option<Stream> {
        if request.uri == self.ws_route {
            websocket_handler(|stream, _| self.streams.lock().unwrap().push(stream))(
                request.clone(),
                stream,
                state.clone(),
            );

            state.logger.info(format!(
                "{}: Hot Reload WebSocket connection opened",
                request.address
            ));

            None
        } else {
            Some(stream)
        }
    }

    fn on_response(&self, response: &mut Response, _: Arc<AppState>, route: &RouteConfig) {
        let content_type = response.headers.get("Content-Type");

        if response.status_code == StatusCode::OK && content_type == Some("text/html") {
            injector::inject_variables(response, route, &self.ws_route);
            injector::inject_js(response, INJECTED_JS);
        } else if content_type
            .map(|s| s.starts_with("image/") || s == "text/javascript" || s == "text/css")
            .unwrap_or(false)
        {
            response.headers.remove("Cache-Control");
            response.headers.add("Cache-Control", "no-store");
        }
    }
}

impl Debug for HotReloadPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HotReloadPlugin").finish()
    }
}

declare_plugin!(HotReloadPlugin, HotReloadPlugin::default);
