use crate::listen::get_url_prefix;

use humphrey::http::Response;
use humphrey_server::config::RouteConfig;

pub fn inject_js(response: &mut Response, js: &[u8]) {
    if let Some(index) = response.body.windows(7).position(|w| w == b"</body>") {
        let mut to_inject = Vec::with_capacity(js.len() + 17);
        to_inject.extend_from_slice(b"<script>");
        to_inject.extend_from_slice(js);
        to_inject.extend_from_slice(b"</script>");

        response.body.splice(index..index, to_inject);
    }
}

pub fn inject_variables(response: &mut Response, route: &RouteConfig, ws_route: &str) {
    if let Some(index) = response.body.windows(7).position(|w| w == b"</body>") {
        response.body.splice(
            index..index,
            format!(
                r#"<script>
                    const __HUMPHREY_ROUTE_PATH = "{}";
                    const __HUMPHREY_ROUTE_URL_PREFIX = "{}";
                    const __HUMPHREY_WS_ROUTE = "{}";
                </script>"#,
                route.path.as_ref().unwrap(),
                get_url_prefix(&route.matches).unwrap(),
                ws_route
            )
            .as_bytes()
            .to_vec(),
        );
    }
}
