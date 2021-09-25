#![allow(unused_imports)]
use crate::config::{load_config, parse_ini, BlacklistMode, Config, ServerMode};
use crate::logger::LogLevel;
use std::collections::HashMap;

#[test]
fn test_parse_ini() {
    let testcase =
        "[s1]\nkey1=value \nkey2 = value2   ; comment\nkey3 = \"value\"\n\n[s2]\nkey1=value";

    let mut expected_hashmap: HashMap<String, String> = HashMap::new();
    expected_hashmap.insert("s1.key1".into(), "value".into());
    expected_hashmap.insert("s1.key2".into(), "value2".into());
    expected_hashmap.insert("s1.key3".into(), "value".into());
    expected_hashmap.insert("s2.key1".into(), "value".into());

    let hashmap = parse_ini(testcase).unwrap();
    assert_eq!(hashmap, expected_hashmap);
}

#[test]
fn test_parse_config() {
    let config_string = r#"
; Humphrey Configuration File

[server]
address = "127.0.0.1"      ; address to host the server on
port = 8000                ; port to host the server on
mode = "static"            ; server routing mode
threads = 123              ; threads to start

[log]
level = "info"
console = false
file = "humphrey.log"

[blacklist]
mode = "forbidden"

[static]
directory = "/var/www"
websocket = "localhost:1234"
cache = 128M
cache_time = 60"#;

    let config = load_config(Some(config_string.into())).unwrap();

    let mut expected_hashmap: HashMap<String, String> = HashMap::new();
    expected_hashmap.insert("server.address".into(), "127.0.0.1".into());
    expected_hashmap.insert("server.port".into(), "8000".into());
    expected_hashmap.insert("server.mode".into(), "static".into());
    expected_hashmap.insert("server.threads".into(), "123".into());
    expected_hashmap.insert("log.level".into(), "info".into());
    expected_hashmap.insert("log.console".into(), "false".into());
    expected_hashmap.insert("log.file".into(), "humphrey.log".into());
    expected_hashmap.insert("blacklist.mode".into(), "forbidden".into());
    expected_hashmap.insert("static.directory".into(), "/var/www".into());
    expected_hashmap.insert("static.websocket".into(), "localhost:1234".into());
    expected_hashmap.insert("static.cache".into(), "128M".into());
    expected_hashmap.insert("static.cache_time".into(), "60".into());

    let expected_config = Config {
        address: "127.0.0.1".into(),
        port: 8000,
        threads: 123,
        mode: ServerMode::Static,
        blacklist: Vec::new(),
        blacklist_mode: BlacklistMode::Forbidden,
        log_level: LogLevel::Info,
        log_console: false,
        log_file: Some("humphrey.log".into()),
        cache_limit: 128 * 1024 * 1024,
        cache_time_limit: 60,
        directory: Some("/var/www".into()),
        websocket_proxy: Some("localhost:1234".into()),
        #[cfg(feature = "plugins")]
        plugin_libraries: Vec::new(),
        proxy_target: None,
        load_balancer_targets: None,
        load_balancer_mode: None,
        raw: expected_hashmap,
    };

    assert_eq!(config, expected_config);
}
