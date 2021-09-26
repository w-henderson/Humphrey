#![allow(unused_imports)]
#![allow(dead_code)]

use crate::config::tree::*;
use std::collections::{BTreeMap, HashMap};

pub const CONF: &'static str = r#"server {
    address   "0.0.0.0"
    port      80
    threads   32

    plugins { # this is a comment on a section header
        php {
            library   "plugins/php/target/release/php.dll"
            address   "127.0.0.1"
            port      9000
            threads   8
        }
    }

    # this is a comment on an empty line

    blacklist {
        # no blacklist file because it's easier for testing
        mode   "block"
    }

    log {
        level     "info"
        console   true
        file      "humphrey.log"
    }

    cache {
        size   128M # this is a comment on a value
        time   60
    }

    route /static/* { # this is a comment on a route header
        directory   "/var/www"
        websocket   "localhost:1234"
    }

    route /* {
        proxy               "127.0.0.1:8000,127.0.0.1:8080"
        load_balancer_mode  "round-robin"
    }
}"#;

#[test]
fn test_build_tree() {
    #[rustfmt::skip]
    let expected_parsed_conf = ConfigNode::Section("server".into(), vec![
        ConfigNode::String("address".into(), "0.0.0.0".into()),
        ConfigNode::Number("port".into(), "80".into()),
        ConfigNode::Number("threads".into(), "32".into()),
        ConfigNode::Section("plugins".into(), vec![
            ConfigNode::Section("php".into(), vec![
                ConfigNode::String("library".into(), "plugins/php/target/release/php.dll".into()),
                ConfigNode::String("address".into(), "127.0.0.1".into()),
                ConfigNode::Number("port".into(), "9000".into()),
                ConfigNode::Number("threads".into(), "8".into())
            ])
        ]),
        ConfigNode::Section("blacklist".into(), vec![
            ConfigNode::String("mode".into(), "block".into()),
        ]),
        ConfigNode::Section("log".into(), vec![
            ConfigNode::String("level".into(), "info".into()),
            ConfigNode::Boolean("console".into(), "true".into()),
            ConfigNode::String("file".into(), "humphrey.log".into()),
        ]),
        ConfigNode::Section("cache".into(), vec![
            ConfigNode::Number("size".into(), "134217728".into()),
            ConfigNode::Number("time".into(), "60".into())
        ]),
        ConfigNode::Route("/static/*".into(), vec![
            ConfigNode::String("directory".into(), "/var/www".into()),
            ConfigNode::String("websocket".into(), "localhost:1234".into()),
        ]),
        ConfigNode::Route("/*".into(), vec![
            ConfigNode::String("proxy".into(), "127.0.0.1:8000,127.0.0.1:8080".into()),
            ConfigNode::String("load_balancer_mode".into(), "round-robin".into())
        ])
    ]);

    let parsed_conf = parse_conf(CONF).unwrap();

    assert_eq!(parsed_conf, expected_parsed_conf);
}

#[test]
#[rustfmt::skip]
fn test_flatten_config() {
    let parsed_conf = parse_conf(CONF).unwrap();

    let mut expected_hashmap: HashMap<String, ConfigNode> = HashMap::new();
    expected_hashmap.insert("server.address".into(), ConfigNode::String("address".into(), "0.0.0.0".into()));
    expected_hashmap.insert("server.port".into(), ConfigNode::Number("port".into(), "80".into()));
    expected_hashmap.insert("server.threads".into(), ConfigNode::Number("threads".into(), "32".into()));
    expected_hashmap.insert("server.blacklist.mode".into(), ConfigNode::String("mode".into(), "block".into()));
    expected_hashmap.insert("server.log.level".into(), ConfigNode::String("level".into(), "info".into()));
    expected_hashmap.insert("server.log.console".into(), ConfigNode::Boolean("console".into(), "true".into()));
    expected_hashmap.insert("server.log.file".into(), ConfigNode::String("file".into(), "humphrey.log".into()));
    expected_hashmap.insert("server.cache.size".into(), ConfigNode::Number("size".into(), "134217728".into()));
    expected_hashmap.insert("server.cache.time".into(), ConfigNode::Number("time".into(), "60".into()));

    let mut actual_hashmap: HashMap<String, ConfigNode> = HashMap::new();
    parsed_conf.flatten(&mut actual_hashmap, &Vec::new());

    assert_eq!(actual_hashmap, expected_hashmap);
}

#[test]
#[rustfmt::skip]
fn test_get_routes() {
    let parsed_conf = parse_conf(CONF).unwrap();

    let mut expected_map: Vec<(String, HashMap<String, ConfigNode>)> = Vec::new();

    let mut static_hashmap: HashMap<String, ConfigNode> = HashMap::new();
    static_hashmap.insert("directory".into(), ConfigNode::String("directory".into(), "/var/www".into()));
    static_hashmap.insert("websocket".into(), ConfigNode::String("websocket".into(), "localhost:1234".into()));
    expected_map.push(("/static/*".into(), static_hashmap));

    let mut proxy_hashmap: HashMap<String, ConfigNode> = HashMap::new();
    proxy_hashmap.insert("proxy".into(), ConfigNode::String("proxy".into(), "127.0.0.1:8000,127.0.0.1:8080".into()));
    proxy_hashmap.insert("load_balancer_mode".into(), ConfigNode::String("load_balancer_mode".into(), "round-robin".into()));
    expected_map.push(("/*".into(), proxy_hashmap));

    let routes = parsed_conf.get_routes();

    assert_eq!(routes, expected_map);
}

#[test]
#[rustfmt::skip]
fn test_get_plugins() {
    let parsed_conf = parse_conf(CONF).unwrap();

    let mut expected_map: Vec<(String, HashMap<String, ConfigNode>)> = Vec::new();

    let mut php_hashmap: HashMap<String, ConfigNode> = HashMap::new();
    php_hashmap.insert("library".into(), ConfigNode::String("library".into(), "plugins/php/target/release/php.dll".into()));
    php_hashmap.insert("address".into(), ConfigNode::String("address".into(), "127.0.0.1".into()));
    php_hashmap.insert("port".into(), ConfigNode::Number("port".into(), "9000".into()));
    php_hashmap.insert("threads".into(), ConfigNode::Number("threads".into(), "8".into()));
    expected_map.push(("php".into(), php_hashmap));

    let plugins = parsed_conf.get_plugins();

    assert_eq!(plugins, expected_map);
}
