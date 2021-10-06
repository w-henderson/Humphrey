use crate::config::{
    BlacklistConfig, BlacklistMode, CacheConfig, Config, LoggingConfig, RouteConfig,
};
use crate::server::logger::LogLevel;

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".into(),
            port: 80,
            threads: 32,
            websocket_proxy: None,
            routes: vec![Default::default()],
            #[cfg(feature = "plugins")]
            plugins: Vec::new(),
            logging: Default::default(),
            cache: Default::default(),
            blacklist: Default::default(),
        }
    }
}

impl Default for RouteConfig {
    fn default() -> Self {
        Self::Serve {
            matches: "/*".into(),
            directory: '.'.into(),
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

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            size_limit: Default::default(),
            time_limit: Default::default(),
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
