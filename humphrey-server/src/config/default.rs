//! Provides default values for the configuration.

use crate::config::{
    BlacklistConfig, BlacklistMode, Config, ConfigSource, HostConfig, LoggingConfig, RouteConfig,
    RouteType,
};
use crate::server::logger::LogLevel;

impl Default for Config {
    fn default() -> Self {
        Self {
            source: ConfigSource::Default,
            address: "0.0.0.0".into(),
            port: 80,
            threads: 32,
            #[cfg(feature = "tls")]
            tls_config: None,
            default_websocket_proxy: None,
            hosts: Vec::new(),
            default_host: Default::default(),
            #[cfg(feature = "plugins")]
            plugins: Vec::new(),
            logging: Default::default(),
            cache: Default::default(),
            blacklist: Default::default(),
            connection_timeout: Default::default(),
        }
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            matches: "*".into(),
            routes: vec![Default::default()],
        }
    }
}

impl Default for RouteConfig {
    fn default() -> Self {
        Self {
            route_type: RouteType::Directory,
            matches: "/*".into(),
            path: Some('.'.into()),
            load_balancer: None,
            websocket_proxy: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            console: true,
            file: None,
        }
    }
}

impl Default for BlacklistConfig {
    fn default() -> Self {
        Self {
            list: Default::default(),
            mode: BlacklistMode::Block,
        }
    }
}
