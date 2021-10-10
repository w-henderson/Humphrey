use humphrey_server::config::error::ConfigError;
use humphrey_server::config::tree::parse_conf;

#[test]
fn value_error() {
    let string = include_str!("./testcases/value_error.conf");
    let config = parse_conf(string, "value_error.conf");

    assert_eq!(
        config,
        Err(ConfigError::new(
            "Could not parse value",
            "value_error.conf",
            34
        ))
    );
}

#[test]
fn eof_error() {
    let string = include_str!("./testcases/eof_error.conf");
    let config = parse_conf(string, "eof_error.conf");

    assert_eq!(
        config,
        Err(ConfigError::new(
            "Unexpected end of file, expected `}`",
            "eof_error.conf",
            44
        ))
    );
}

#[test]
fn empty_file_error() {
    let string = include_str!("./testcases/empty_file_error.conf");
    let config = parse_conf(string, "empty_file_error.conf");

    assert_eq!(
        config,
        Err(ConfigError::new(
            "Could not find `server` section",
            "empty_file_error.conf",
            0
        ))
    );
}
