use humphrey_server::config::tree::parse_conf;
use humphrey_server::config::{
    BlacklistConfig, BlacklistMode, CacheConfig, Config, ConfigSource, HostConfig,
    LoadBalancerMode, LoggingConfig, RouteConfig, RouteType,
};
use humphrey_server::logger::LogLevel;
use humphrey_server::proxy::{EqMutex, LoadBalancer};
use humphrey_server::rand::Lcg;

use std::env::set_current_dir;
use std::path::Path;

#[test]
fn include_route() {
    // Set current directory to testcases directory so the parser can find the included file
    let testcases_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/testcases");
    set_current_dir(testcases_path).unwrap();

    let string = include_str!("./testcases/include_route.conf");
    let config = Config::from_tree(parse_conf(string, "include_route.conf").unwrap());

    let expected_conf = Ok(Config {
        source: ConfigSource::Default,
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
    });

    assert_eq!(config, expected_conf);
}

#[test]
fn nested_include() {
    // Set current directory to testcases directory so the parser can find the included files
    let testcases_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/testcases");
    set_current_dir(testcases_path).unwrap();

    let string = include_str!("./testcases/nested_include_root.conf");
    let config = Config::from_tree(parse_conf(string, "nested_include_root.conf").unwrap());

    let expected_conf = Ok(Config {
        source: ConfigSource::Default,
        address: "0.0.0.0".into(),
        port: 80,
        threads: 32,
        #[cfg(feature = "tls")]
        tls_config: None,
        default_websocket_proxy: None,
        default_host: HostConfig {
            matches: "*".into(),
            routes: vec![RouteConfig {
                route_type: RouteType::Proxy,
                matches: "/test".into(),
                path: None,
                load_balancer: Some(EqMutex::new(LoadBalancer {
                    targets: vec!["127.0.0.1".into()],
                    mode: LoadBalancerMode::Random,
                    index: 0,
                    lcg: Lcg::new(),
                })),
                websocket_proxy: None,
            }],
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
    });

    assert_eq!(config, expected_conf);
}
