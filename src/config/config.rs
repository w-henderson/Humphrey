use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::Read;

use crate::config::extended_hashmap::ExtendedMap;
use crate::logger::LogLevel;
use humphrey::krauss::wildcard_match;

/// Stores the server configuration.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    /// Address to host the server on
    pub address: String,
    /// Port to host the server on
    pub port: u16,
    /// Routing mode of the server
    pub mode: ServerMode,
    /// Blacklisted IP addresses
    pub blacklist: Vec<String>,
    /// Mode of blacklisting
    pub blacklist_mode: BlacklistMode,
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
    /// WebSocket proxy address
    pub websocket_proxy: Option<String>,
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

/// Represents a method of applying the blacklist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlacklistMode {
    /// Does not allow any access from blacklisted addresses
    Block,
    /// Returns 400 Forbidden to every request, only available in static mode
    Forbidden,
}

impl Default for LoadBalancerMode {
    fn default() -> Self {
        Self::RoundRobin
    }
}

impl Default for BlacklistMode {
    fn default() -> Self {
        Self::Block
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".into(),
            port: 80,
            mode: ServerMode::Static,
            blacklist: Vec::new(),
            blacklist_mode: BlacklistMode::Block,
            log_level: LogLevel::Warn,
            log_console: true,
            log_file: None,
            cache_limit: 0,
            cache_time_limit: 0,
            directory: Some(String::new()),
            websocket_proxy: None,
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
    let mode = hashmap.get_compulsory("server.mode", "The mode was not specified")?;

    // Get and validate the specified address and port
    let address = hashmap.get_optional("server.address", "0.0.0.0".into());
    let port: u16 = hashmap.get_optional_parsed("server.port", 80, "Invalid port")?;

    // Get and validate the blacklist file
    let blacklist = load_blacklist(hashmap.get("blacklist.file".into()).clone())?;

    // Read the blacklist mode
    let blacklist_mode = hashmap.get_optional("blacklist.mode", "block".into());

    // Parse the blacklist mode
    let blacklist_mode = match blacklist_mode.as_str() {
        "block" => BlacklistMode::Block,
        "forbidden" => BlacklistMode::Forbidden,
        _ => return Err("The blacklist mode was invalid"),
    };

    if mode != "static" && blacklist_mode == BlacklistMode::Forbidden {
        return Err("Blacklist mode 'forbidden' is only supported when serving static content");
    }

    // Get logging configuration
    let log_level =
        hashmap.get_optional_parsed("log.level", LogLevel::Warn, "Invalid log level")?;
    let log_file = hashmap.get_owned("log.file");
    let log_console = hashmap.get("log.console".into()) != Some(&String::from("false"));

    match mode.as_str() {
        "static" => {
            // Get and parse the cache size limit
            let cache_size_limit = hashmap.get_optional("static.cache", "0".into());
            let cache_size_limit = parse_size(cache_size_limit)?;

            // Get and parse the cache time limit
            let cache_time_limit: u64 =
                hashmap.get_optional_parsed("static.cache_time", 60, "Invalid cache time limit")?;

            // Get the root directory
            let directory = hashmap.get_optional("static.directory", String::new());

            // Get the WebSocket proxy address
            let websocket_proxy = hashmap.get("static.websocket").map(|s| s.to_string());

            Ok(Config {
                address,
                port,
                mode: ServerMode::Static,
                blacklist,
                blacklist_mode,
                log_level,
                log_console,
                log_file,
                cache_limit: cache_size_limit,
                cache_time_limit,
                directory: Some(directory),
                websocket_proxy,
                proxy_target: None,
                load_balancer_targets: None,
                load_balancer_mode: None,
            })
        }
        "proxy" => {
            let proxy_target =
                hashmap.get_compulsory("proxy.target", "The proxy target was not specified")?;

            Ok(Config {
                address,
                port,
                mode: ServerMode::Proxy,
                blacklist,
                blacklist_mode,
                log_level,
                log_console,
                log_file,
                cache_limit: 0,
                cache_time_limit: 0,
                directory: None,
                websocket_proxy: None,
                proxy_target: Some(proxy_target.clone()),
                load_balancer_targets: None,
                load_balancer_mode: None,
            })
        }
        "load_balancer" => {
            // Try to get the path to the targets file
            let targets_file = hashmap.get_compulsory(
                "load_balancer.targets",
                "The load balancer targets file was not specified",
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
            let load_balancer_mode =
                hashmap.get_optional("load_balancer.mode", "round-robin".into());

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
                blacklist,
                blacklist_mode,
                log_level,
                log_console,
                log_file,
                cache_limit: 0,
                cache_time_limit: 0,
                directory: None,
                websocket_proxy: None,
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
        // The file can be opened

        let mut string = String::new();
        if let Ok(_) = file.read_to_string(&mut string) {
            // The file can be read

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
pub fn parse_ini(ini: &str) -> Result<HashMap<String, String>, ()> {
    let mut config: HashMap<String, String> = HashMap::new();
    let mut section: Option<String> = None;

    for line in ini.lines() {
        if line.chars().nth(0) == Some(';') || line.len() == 0 {
            // If the line is empty or a comment, ignore it
            continue;
        }

        if wildcard_match("[*]", line) {
            // If the line is a section header, set the section and continue
            section = Some(line[1..line.len() - 1].to_string());
            continue;
        }

        // Get the key and the value separated by the `:`
        let line = line.splitn(2, ';').nth(0).unwrap();
        let mut key_value = line.splitn(2, '=');
        let key = key_value.next();
        let value = key_value.next();

        if key.is_some() && value.is_some() {
            // Both the key and the value are valid

            // If the value is in quotation marks, remove them
            let value = value.unwrap().trim();
            let value = if wildcard_match("\"*\"", value) {
                &value[1..value.len() - 1]
            } else {
                value
            };

            // If currently in a section, prepend the section name and a dot to the key
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
        // Empty string

        Err("The specified size was invalid")
    } else if size.len() == 1 {
        // One character so cannot possibly be valid

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

/// Loads the blacklist file.
fn load_blacklist(path: Option<&String>) -> Result<Vec<String>, &'static str> {
    if let Some(path) = path {
        // Try to open and read the file
        let mut file = File::open(path).map_err(|_| "Blacklist file could not be opened")?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|_| "Blacklist file could not be read")?;

        // Collect the lines of the file into a `Vec`
        let blacklist: Vec<String> = buf.lines().map(|s| s.to_string()).collect();

        Ok(blacklist)
    } else {
        // Return an empty `Vec` if no file was supplied
        Ok(Vec::new())
    }
}
