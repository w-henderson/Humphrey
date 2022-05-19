//! Provides heartbeat (ping/pong) configuration utilities.

use std::time::Duration;

/// Represents heartbeat configuration.
///
/// A heartbeat, also known as a ping/pong, is a mechanism that allows the server to ensure that the
///   client is still connected and functioning properly. If enabled, the server will send a small message
///   to the client every `interval` seconds. If the client does not respond to any pings within
///   `timeout` seconds, the server will close the connection and run the configured callbacks.
///
/// You can learn more about heartbeats on the [MDN docs](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers#pings_and_pongs_the_heartbeat_of_websockets).
///
/// This struct implements `Default`, defaulting to a heartbeat interval of 5 seconds and a timeout of 10 seconds.
pub struct Heartbeat {
    pub(crate) interval: Duration,
    pub(crate) timeout: Duration,
}

impl Heartbeat {
    /// Create a new heartbeat configuration with the given interval and timeout.
    pub fn new(interval: Duration, timeout: Duration) -> Self {
        Self { interval, timeout }
    }
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5),
            timeout: Duration::from_secs(10),
        }
    }
}
