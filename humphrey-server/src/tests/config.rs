#![allow(unused_imports)]
#![allow(dead_code)]

use crate::config::tree::*;
use std::collections::HashMap;

const CONF: &'static str = r#"server {
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
        file   "conf/blacklist.txt"
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
        proxy               "http://127.0.0.1:8000,http://127.0.0.1:8080"
        load_balance_mode   "round-robin"
    }
}"#;

#[test]
fn test_conf_parse() {
    #[rustfmt::skip]
    let expected_parsed_conf = ConfigNode::Section("server".into(), vec![
        ConfigNode::String("address".into(), "0.0.0.0".into()),
        ConfigNode::Number("port".into(), 80),
        ConfigNode::Number("threads".into(), 32),
        ConfigNode::Section("plugins".into(), vec![
            ConfigNode::Section("php".into(), vec![
                ConfigNode::String("library".into(), "plugins/php/target/release/php.dll".into()),
                ConfigNode::String("address".into(), "127.0.0.1".into()),
                ConfigNode::Number("port".into(), 9000),
                ConfigNode::Number("threads".into(), 8)
            ])
        ]),
        ConfigNode::Section("blacklist".into(), vec![
            ConfigNode::String("file".into(), "conf/blacklist.txt".into()),
            ConfigNode::String("mode".into(), "block".into()),
        ]),
        ConfigNode::Section("log".into(), vec![
            ConfigNode::String("level".into(), "info".into()),
            ConfigNode::Boolean("console".into(), true),
            ConfigNode::String("file".into(), "humphrey.log".into()),
        ]),
        ConfigNode::Section("cache".into(), vec![
            ConfigNode::Number("size".into(), 0x8000000),
            ConfigNode::Number("time".into(), 60)
        ]),
        ConfigNode::Route("/static/*".into(), vec![
            ConfigNode::String("directory".into(), "/var/www".into()),
            ConfigNode::String("websocket".into(), "localhost:1234".into()),
        ]),
        ConfigNode::Route("/*".into(), vec![
            ConfigNode::String("proxy".into(), "http://127.0.0.1:8000,http://127.0.0.1:8080".into()),
            ConfigNode::String("load_balance_mode".into(), "round-robin".into())
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
    expected_hashmap.insert("server.port".into(), ConfigNode::Number("port".into(), 80));
    expected_hashmap.insert("server.threads".into(), ConfigNode::Number("threads".into(), 32));
    expected_hashmap.insert("server.plugins.php.library".into(), ConfigNode::String("library".into(), "plugins/php/target/release/php.dll".into()));
    expected_hashmap.insert("server.plugins.php.address".into(), ConfigNode::String("address".into(), "127.0.0.1".into()));
    expected_hashmap.insert("server.plugins.php.port".into(), ConfigNode::Number("port".into(), 9000));
    expected_hashmap.insert("server.plugins.php.threads".into(), ConfigNode::Number("threads".into(), 8));
    expected_hashmap.insert("server.blacklist.file".into(), ConfigNode::String("file".into(), "conf/blacklist.txt".into()));
    expected_hashmap.insert("server.blacklist.mode".into(), ConfigNode::String("mode".into(), "block".into()));
    expected_hashmap.insert("server.log.level".into(), ConfigNode::String("level".into(), "info".into()));
    expected_hashmap.insert("server.log.console".into(), ConfigNode::Boolean("console".into(), true));
    expected_hashmap.insert("server.log.file".into(), ConfigNode::String("file".into(), "humphrey.log".into()));
    expected_hashmap.insert("server.cache.size".into(), ConfigNode::Number("size".into(), 0x8000000));
    expected_hashmap.insert("server.cache.time".into(), ConfigNode::Number("time".into(), 60));

    let mut actual_hashmap: HashMap<String, ConfigNode> = HashMap::new();
    parsed_conf.flatten(&mut actual_hashmap, &Vec::new());

    assert_eq!(actual_hashmap, expected_hashmap);
}
