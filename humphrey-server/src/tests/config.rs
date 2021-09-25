#[allow(unused_imports)]
use crate::config::config::*;

#[test]
fn test_conf_parse() {
    let conf = r#"server {
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

    #[rustfmt::skip]
    let expected_parsed_conf = ConfigValue::Section("server".into(), vec![
        ConfigValue::String("address".into(), "0.0.0.0".into()),
        ConfigValue::Number("port".into(), 80),
        ConfigValue::Number("threads".into(), 32),
        ConfigValue::Section("plugins".into(), vec![
            ConfigValue::Section("php".into(), vec![
                ConfigValue::String("library".into(), "plugins/php/target/release/php.dll".into()),
                ConfigValue::String("address".into(), "127.0.0.1".into()),
                ConfigValue::Number("port".into(), 9000),
                ConfigValue::Number("threads".into(), 8)
            ])
        ]),
        ConfigValue::Section("blacklist".into(), vec![
            ConfigValue::FilePath("file".into(), "conf/blacklist.txt".into()),
            ConfigValue::String("mode".into(), "block".into()),
        ]),
        ConfigValue::Section("log".into(), vec![
            ConfigValue::String("level".into(), "info".into()),
            ConfigValue::Boolean("console".into(), true),
            ConfigValue::FilePath("file".into(), "humphrey.log".into()),
        ]),
        ConfigValue::Section("cache".into(), vec![
            ConfigValue::Number("size".into(), 0x8000000),
            ConfigValue::Number("time".into(), 60)
        ]),
        ConfigValue::Route("/static/*".into(), vec![
            ConfigValue::String("directory".into(), "/var/www".into()),
            ConfigValue::String("websocket".into(), "localhost:1234".into()),
        ]),
        ConfigValue::Route("/*".into(), vec![
            ConfigValue::String("proxy".into(), "http://127.0.0.1:8000,http://127.0.0.1:8080".into()),
            ConfigValue::String("load_balance_mode".into(), "round-robin".into())
        ])
    ]);

    let parsed_conf = parse_conf(conf).unwrap();

    assert_eq!(parsed_conf, expected_parsed_conf);
}
