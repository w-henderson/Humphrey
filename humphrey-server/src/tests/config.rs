#![allow(unused_imports)]
use super::tree::CONF;
use crate::config::config::{
    BlacklistConfig, BlacklistMode, CacheConfig, Config, LoadBalancerMode, LoggingConfig,
    RouteConfig,
};
use crate::config::tree::{parse_conf, ConfigNode};
use crate::logger::LogLevel;

#[cfg(feature = "plugins")]
use crate::config::config::PluginConfig;

use std::collections::HashMap;

#[test]
fn test_parse_config() {
    let tree = parse_conf(CONF).unwrap();
    let conf = Config::from_tree(tree).unwrap();

    #[cfg(feature = "plugins")]
    let expected_plugin_conf = {
        let mut map = HashMap::new();
        map.insert("address".into(), "127.0.0.1".into());
        map.insert("port".into(), "9000".into());
        map.insert("threads".into(), "8".into());
        map
    };

    let expected_conf = Config {
        address: "0.0.0.0".into(),
        port: 80,
        threads: 32,
        routes: vec![
            RouteConfig::Serve {
                matches: "/static/*".into(),
                directory: "/var/www".into(),
                websocket_proxy: Some("localhost:1234".into()),
            },
            RouteConfig::Proxy {
                matches: "/*".into(),
                targets: vec!["127.0.0.1:8000".into(), "127.0.0.1:8080".into()],
                load_balancer_mode: LoadBalancerMode::RoundRobin,
            },
        ],
        #[cfg(feature = "plugins")]
        plugins: vec![PluginConfig {
            name: "php".into(),
            library: "plugins/php/target/release/php.dll".into(),
            config: expected_plugin_conf,
        }],
        logging: LoggingConfig {
            level: LogLevel::Info,
            console: true,
            file: Some("humphrey.log".into()),
        },
        cache: CacheConfig {
            size_limit: 134217728,
            time_limit: 60,
        },
        blacklist: BlacklistConfig {
            list: vec![],
            mode: BlacklistMode::Block,
        },
    };

    assert_eq!(conf, expected_conf);
}
