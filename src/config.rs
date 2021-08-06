use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::Read;

use humphrey::krauss::wildcard_match;

pub struct Config {
    pub address: String,
    pub port: u16,
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
