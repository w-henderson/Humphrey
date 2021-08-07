use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::Read;

use humphrey::krauss::wildcard_match;

#[path = "tests/config.rs"]
mod tests;

/// Stores the server configuration.
pub struct Config {
    /// Address to host the server on
    pub address: String,
    /// Port to host the server on
    pub port: u16,
    /// Routing mode of the server
    pub mode: ServerMode,
    /// Root directory to serve files from
    pub directory: Option<String>,
    /// Proxy target address
    pub proxy_target: Option<String>,
    /// Targets for the load balancer
    pub load_balancer_targets: Option<Vec<String>>,
    /// Algorithm for load balancing
    pub load_balancer_mode: Option<LoadBalancerMode>,
}

// Represents a hosting mode.
pub enum ServerMode {
    /// Host static content from a directory
    Static,
    /// Proxy requests to another server
    Proxy,
    /// Distribute requests to a number of different servers
    LoadBalancer,
}

/// Represents an algorithm for load balancing.
pub enum LoadBalancerMode {
    RoundRobin,
    Random,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".into(),
            port: 80,
            mode: ServerMode::Static,
            directory: Some(String::new()),
            proxy_target: None,
            load_balancer_targets: None,
            load_balancer_mode: None,
        }
    }
}

/// Locates, parses and returns the server configuration.
/// Returns `Err` if any part of this process fails.
pub fn load_config() -> Result<Config, &'static str> {
    if let Ok(config_string) = read_config() {
        // The configuration was read successfully

        if let Ok(hashmap) = parse_ini(&config_string) {
            // The configuration was parsed successfully

            if let Ok(port) = hashmap
                .get("server.port".into())
                .unwrap_or(&"80".into())
                .parse::<u16>()
            {
                // The port was a valid `u16` number

                if let Some(mode) = hashmap.get("server.mode".into()) {
                    // The mode was specified

                    if mode == "static" {
                        Ok(Config {
                            address: hashmap
                                .get("server.address".into())
                                .unwrap_or(&"0.0.0.0".into())
                                .to_string(),
                            port,
                            mode: ServerMode::Static,
                            directory: Some(
                                hashmap
                                    .get("static.directory".into())
                                    .unwrap_or(&String::new())
                                    .to_string(),
                            ),
                            proxy_target: None,
                            load_balancer_targets: None,
                            load_balancer_mode: None,
                        })
                    } else if mode == "proxy" {
                        if let Some(proxy_target) = hashmap.get("proxy.target") {
                            // The proxy target was specified

                            Ok(Config {
                                address: hashmap
                                    .get("server.address".into())
                                    .unwrap_or(&"0.0.0.0".into())
                                    .to_string(),
                                port,
                                mode: ServerMode::Proxy,
                                directory: None,
                                proxy_target: Some(proxy_target.clone()),
                                load_balancer_targets: None,
                                load_balancer_mode: None,
                            })
                        } else {
                            Err("The proxy target was not specified")
                        }
                    } else if mode == "load_balancer" {
                        if let Some(targets_file) = hashmap.get("load_balancer.targets") {
                            // The load balancer targets file was specified

                            if let Ok(mut targets_file) = File::open(targets_file) {
                                // The load balancer targets file was successfully opened

                                let mut buf = String::new();
                                if let Ok(_) = targets_file.read_to_string(&mut buf) {
                                    // The load balancer targets file was successfully read

                                    let load_balancer_mode = hashmap
                                        .get("load_balancer.mode".into())
                                        .unwrap_or(&"round-robin".into())
                                        .to_string();

                                    // Parse the load balancer mode
                                    let load_balancer_mode = match load_balancer_mode.as_str() {
                                        "round-robin" => LoadBalancerMode::RoundRobin,
                                        "random" => LoadBalancerMode::Random,
                                        _ => return Err("The load balancer mode was invalid"),
                                    };

                                    let targets: Vec<String> =
                                        buf.lines().map(|s| s.to_string()).collect();

                                    Ok(Config {
                                        address: hashmap
                                            .get("server.address".into())
                                            .unwrap_or(&"0.0.0.0".into())
                                            .to_string(),
                                        port,
                                        mode: ServerMode::LoadBalancer,
                                        directory: None,
                                        proxy_target: None,
                                        load_balancer_targets: Some(targets),
                                        load_balancer_mode: Some(load_balancer_mode),
                                    })
                                } else {
                                    Err("The load balancer targets file could not be read")
                                }
                            } else {
                                Err("The load balancer targets file could not be opened")
                            }
                        } else {
                            Err("The load balancer targets file was not specified")
                        }
                    } else {
                        Err("The server mode was invalid")
                    }
                } else {
                    Err("The server mode was not specified")
                }
            } else {
                Err("The server port is invalid")
            }
        } else {
            Err("The configuration file could not be parsed")
        }
    } else {
        Err("The configuration file could not be found")
    }
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
    let mut section: Option<String> = None;

    for line in ini.lines() {
        if line.chars().nth(0) == Some(';') || line.len() == 0 {
            continue;
        }

        if wildcard_match("[*]", line) {
            section = Some(line[1..line.len() - 1].to_string());
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

            let full_key = if let Some(s) = &section {
                format!("{}.{}", s, key.unwrap().trim())
            } else {
                key.unwrap().trim().into()
            };

            config.insert(full_key, value.into());
        } else {
            return Err(());
        }
    }

    Ok(config)
}
