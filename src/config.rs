use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::Read;

use humphrey::krauss::wildcard_match;

/// Stores the server configuration.
pub struct Config {
    /// Address to host the server on
    pub address: String,
    /// Port to host the server on
    pub port: u16,
    /// Root directory to serve files from
    pub directory: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".into(),
            port: 80,
            directory: String::new(),
        }
    }
}

/// Locates, parses and returns the server configuration.
/// Returns `Err` if any part of this process fails.
pub fn load_config() -> Result<Config, ()> {
    if let Ok(config_string) = read_config() {
        if let Ok(hashmap) = parse_ini(&config_string) {
            if let Ok(port) = hashmap
                .get("port".into())
                .unwrap_or(&"80".into())
                .parse::<u16>()
            {
                return Ok(Config {
                    address: hashmap
                        .get("address".into())
                        .unwrap_or(&"0.0.0.0".into())
                        .to_string(),
                    port,
                    directory: hashmap
                        .get("directory".into())
                        .unwrap_or(&String::new())
                        .to_string(),
                });
            }
        }
    }

    Err(())
}

/// Locates and reads the configuration file.
/// Uses the first command line argument as a path, defaults to "humphrey.ini" in the current directory if not specified.
/// If the file cannot be found and/or read, returns `Err`.
fn read_config() -> Result<String, ()> {
    let path = args().nth(1).unwrap_or("humphrey.ini".into());

    if let Ok(mut file) = File::open(path) {
        let mut string = String::new();
        if let Ok(_) = file.read_to_string(&mut string) {
            Ok(string)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

/// Attempts to parse the given string as the INI configuration format.
/// If successful, returns a hashmap of keys and values.
/// Otherwise, returns `Err`.
fn parse_ini(ini: &str) -> Result<HashMap<String, String>, ()> {
    let mut config: HashMap<String, String> = HashMap::new();

    for line in ini.lines() {
        if line.chars().nth(0) == Some(';') || line.len() == 0 {
            continue;
        }

        let line = line.splitn(2, ';').nth(0).unwrap();
        let mut key_value = line.splitn(2, '=');
        let key = key_value.next();
        let value = key_value.next();

        if key.is_some() && value.is_some() {
            let value = value.unwrap().trim();
            let value = if wildcard_match("\"*\"", value) {
                &value[1..value.len() - 1]
            } else {
                value
            };

            config.insert(key.unwrap().trim().into(), value.into());
        } else {
            return Err(());
        }
    }

    Ok(config)
}
