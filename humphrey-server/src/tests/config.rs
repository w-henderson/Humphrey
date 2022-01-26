#![allow(unused_imports)]
use super::tree::CONF;
use humphrey_server::config::config::{
    BlacklistConfig, BlacklistMode, CacheConfig, Config, LoadBalancerMode, LoggingConfig,
    RouteConfig,
};
use humphrey_server::config::tree::{parse_conf, ConfigNode};
use humphrey_server::config::{HostConfig, RouteType};
use humphrey_server::logger::LogLevel;

#[cfg(feature = "plugins")]
use humphrey_server::config::config::PluginConfig;
use humphrey_server::proxy::{EqMutex, LoadBalancer};
use humphrey_server::rand::Lcg;

use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_parse_config() {
    let tree = parse_conf(CONF, "valid.conf").unwrap();
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
        #[cfg(feature = "tls")]
        tls_config: None,
        default_websocket_proxy: Some("localhost:1234".into()),
        default_host: HostConfig {
            matches: "*".into(),
            routes: vec![
                RouteConfig {
                    route_type: RouteType::Directory,
                    matches: "/static/*".into(),
                    path: Some("/var/www".into()),
                    load_balancer: None,
                    websocket_proxy: None,
                },
                RouteConfig {
                    route_type: RouteType::Proxy,
                    matches: "/*".into(),
                    path: None,
                    load_balancer: Some(EqMutex::new(LoadBalancer {
                        targets: vec!["127.0.0.1:8000".into(), "127.0.0.1:8080".into()],
                        mode: LoadBalancerMode::RoundRobin,
                        index: 0,
                        lcg: Lcg::new(),
                    })),
                    websocket_proxy: None,
                },
            ],
        },
        hosts: Vec::new(),
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
            list: Vec::new(),
            mode: BlacklistMode::Block,
        },
        connection_timeout: Some(Duration::from_secs(5)),
    };

    assert_eq!(conf, expected_conf);
}

#[test]
fn test_host_config() {
    let tree = parse_conf(include_str!("testcases/hosts.conf"), "hosts.conf").unwrap();
    let conf = Config::from_tree(tree).unwrap();

    let expected_conf = Config {
        address: "0.0.0.0".into(),
        port: 80,
        threads: 32,
        #[cfg(feature = "tls")]
        tls_config: None,
        default_websocket_proxy: None,
        default_host: HostConfig {
            matches: "*".into(),
            routes: vec![RouteConfig {
                route_type: RouteType::Directory,
                matches: "/*".into(),
                path: Some("/var/www".into()),
                load_balancer: None,
                websocket_proxy: None,
            }],
        },
        hosts: vec![
            HostConfig {
                matches: "localhost".into(),
                routes: vec![RouteConfig {
                    route_type: RouteType::Redirect,
                    matches: "/".into(),
                    path: Some("/app/dev".into()),
                    load_balancer: None,
                    websocket_proxy: None,
                }],
            },
            HostConfig {
                matches: "*.example.com".into(),
                routes: vec![RouteConfig {
                    route_type: RouteType::Redirect,
                    matches: "/".into(),
                    path: Some("/app/prod".into()),
                    load_balancer: None,
                    websocket_proxy: None,
                }],
            },
        ],
        #[cfg(feature = "plugins")]
        plugins: Vec::new(),
        logging: LoggingConfig {
            level: LogLevel::Warn,
            console: true,
            file: None,
        },
        cache: CacheConfig {
            size_limit: 0,
            time_limit: 0,
        },
        blacklist: BlacklistConfig {
            list: Vec::new(),
            mode: BlacklistMode::Block,
        },
        connection_timeout: None,
    };

    assert_eq!(conf, expected_conf);
}

#[test]
fn comma_separated_routes() {
    let tree = parse_conf(include_str!("testcases/commas.conf"), "commas.conf").unwrap();
    let conf = Config::from_tree(tree).unwrap();

    let expected_conf = Config {
        address: "0.0.0.0".into(),
        port: 80,
        threads: 32,
        #[cfg(feature = "tls")]
        tls_config: None,
        default_websocket_proxy: None,
        default_host: HostConfig {
            matches: "*".into(),
            routes: vec![
                RouteConfig {
                    route_type: RouteType::Directory,
                    matches: "/example/*".into(),
                    path: Some("/var/www".into()),
                    load_balancer: None,
                    websocket_proxy: None,
                },
                RouteConfig {
                    route_type: RouteType::Directory,
                    matches: "/test/*".into(),
                    path: Some("/var/www".into()),
                    load_balancer: None,
                    websocket_proxy: None,
                },
            ],
        },
        hosts: Vec::new(),
        #[cfg(feature = "plugins")]
        plugins: Vec::new(),
        logging: LoggingConfig {
            level: LogLevel::Warn,
            console: true,
            file: None,
        },
        cache: CacheConfig {
            size_limit: 0,
            time_limit: 0,
        },
        blacklist: BlacklistConfig {
            list: Vec::new(),
            mode: BlacklistMode::Block,
        },
        connection_timeout: None,
    };

    assert_eq!(conf, expected_conf);
}
