use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::Read;

use crate::logger::LogLevel;
use humphrey::krauss::wildcard_match;

#[path = "tests/config.rs"]
mod tests;

/// Stores the server configuration.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    /// Address to host the server on
    pub address: String,
    /// Port to host the server on
    pub port: u16,
    /// Routing mode of the server
    pub mode: ServerMode,
    /// Log level of the server
    pub log_level: LogLevel,
    /// Whether to log to the console
    pub log_console: bool,
    /// Logging output file
    pub log_file: Option<String>,
    /// Size limit of the in-memory file cache, measured in bytes
    pub cache_limit: usize,
    /// Time limit for cached items
    pub cache_time_limit: u64,
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
#[derive(Debug, PartialEq, Eq)]
pub enum ServerMode {
    /// Host static content from a directory
    Static,
    /// Proxy requests to another server
    Proxy,
    /// Distribute requests to a number of different servers
    LoadBalancer,
}

/// Represents an algorithm for load balancing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadBalancerMode {
    /// Evenly distributes load in a repeating pattern
    RoundRobin,
    /// Randomly distributes load
    Random,
}

impl Default for LoadBalancerMode {
    fn default() -> Self {
        Self::RoundRobin
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".into(),
            port: 80,
            mode: ServerMode::Static,
            log_level: LogLevel::Warn,
            log_console: true,
            log_file: None,
            cache_limit: 0,
            cache_time_limit: 0,
            directory: Some(String::new()),
            proxy_target: None,
            load_balancer_targets: None,
            load_balancer_mode: None,
        }
    }
}

/// Locates, parses and returns the server configuration.
/// Returns `Err` if any part of this process fails.
pub fn load_config(config_string: Option<String>) -> Result<Config, &'static str> {
    // Load and parse the configuration
    let config = if config_string.is_some() {
        config_string.unwrap()
    } else if let Ok(config) = read_config() {
        config
    } else {
        return Err("The configuration file could not be found");
    };

    let hashmap = parse_ini(&config).map_err(|_| "The configuration file could not be parsed")?;
    let mode = hashmap
        .get("server.mode".into())
        .map_or(Err("The mode was not specified"), |s| Ok(s))?;

    // Get and validate the specified address and port
    let address = hashmap
        .get("server.address".into())
        .unwrap_or(&"0.0.0.0".into())
        .to_string();
    let port = hashmap
        .get("server.port".into())
        .unwrap_or(&"80".into())
        .parse::<u16>()
        .map_err(|_| "The specified port was invalid")?;

    // Get logging configuration
    let log_level = hashmap
        .get("log.level".into())
        .unwrap_or(&"warn".into())
        .parse::<LogLevel>()?;
    let log_file = hashmap.get("log.file".into()).map(|s| s.clone());
    let log_console = hashmap.get("log.console".into()) != Some(&String::from("false"));

    match mode.as_str() {
        "static" => {
            // Get and parse the cache size limit
            let cache_size_limit = hashmap
                .get("static.cache".into())
                .map_or("0".into(), |s| s.clone());
            let cache_size_limit = parse_size(cache_size_limit.clone())?;

            // Get and parse the cache time limit
            let cache_time_limit = hashmap
                .get("static.cache_time".into())
                .unwrap_or(&"60".into())
                .parse::<u64>()
                .map_err(|_| "The specified cache time limit was invalid")?;

            let directory = hashmap
                .get("static.directory".into())
                .unwrap_or(&String::new())
                .to_string();

            Ok(Config {
                address,
                port,
                mode: ServerMode::Static,
                log_level,
                log_console,
                log_file,
                cache_limit: cache_size_limit,
                cache_time_limit,
                directory: Some(directory),
                proxy_target: None,
                load_balancer_targets: None,
                load_balancer_mode: None,
            })
        }
        "proxy" => {
            let proxy_target = hashmap
                .get("proxy.target")
                .map_or(Err("The proxy target was not specified"), |s| Ok(s.clone()))?;

            Ok(Config {
                address,
                port,
                mode: ServerMode::Proxy,
                log_level,
                log_console,
                log_file,
                cache_limit: 0,
                cache_time_limit: 0,
                directory: None,
                proxy_target: Some(proxy_target.clone()),
                load_balancer_targets: None,
                load_balancer_mode: None,
            })
        }
        "load_balancer" => {
            // Try to get the path to the targets file
            let targets_file = hashmap.get("load_balancer.targets").map_or(
                Err("The load balancer targets file was not specified"),
                |s| Ok(s),
            )?;

            // Try to open the targets file
            let mut targets_file = File::open(targets_file)
                .map_err(|_| "The load balancer targets file could not be opened")?;

            // Try to read the targets file
            let mut buf = String::new();
            targets_file
                .read_to_string(&mut buf)
                .map_err(|_| "The load balancer targets file could not be read")?;

            // Read the load balancer mode
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

            let targets: Vec<String> = buf.lines().map(|s| s.to_string()).collect();

            Ok(Config {
                address,
                port,
                mode: ServerMode::LoadBalancer,
                log_level,
                log_console,
                log_file,
                cache_limit: 0,
                cache_time_limit: 0,
                directory: None,
                proxy_target: None,
                load_balancer_targets: Some(targets),
                load_balancer_mode: Some(load_balancer_mode),
            })
        }
        _ => Err("The server mode was invalid"),
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

/// Parses a size string into its corresponding number of bytes.
/// For example, 4K => 4096, 1M => 1048576.
/// If no letter is provided at the end, assumes the number to be in bytes.
fn parse_size(size: String) -> Result<usize, &'static str> {
    if size.len() == 0 {
        Err("The specified size was invalid")
    } else if size.len() == 1 {
        size.parse::<usize>()
            .map_err(|_| "The specified size was invalid")
    } else {
        let last_char = size.chars().last().unwrap().to_ascii_uppercase();
        let number: usize = size[0..size.len() - 1]
            .parse()
            .map_err(|_| "The specified size was invalid")?;

        match last_char {
            'K' => Ok(number * 1024),
            'M' => Ok(number * 1024 * 1024),
            'G' => Ok(number * 1024 * 1024 * 1024),
            '0'..='9' => size
                .parse::<usize>()
                .map_err(|_| "The specified size was invalid"),
            _ => Err("The specified size was invalid"),
        }
    }
}
