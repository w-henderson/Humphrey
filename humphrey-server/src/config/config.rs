use crate::config::extended_hashmap::ExtendedMap;
use crate::config::tree::{parse_conf, ConfigNode};
use crate::logger::LogLevel;
use crate::proxy::{EqMutex, LoadBalancer};
use crate::rand::Lcg;

use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::Read;

/// Represents the parsed and validated configuration.
#[derive(Debug, PartialEq)]
pub struct Config {
    /// The address to host the server on
    pub address: String,
    /// The port to host the server on
    pub port: u16,
    /// The number of threads to host the server on
    pub threads: usize,
    /// Address to forward WebSocket connections to
    pub websocket_proxy: Option<String>,
    /// The configuration for different routes
    pub routes: Vec<RouteConfig>,
    /// The configuration for any plugins
    #[cfg(feature = "plugins")]
    pub plugins: Vec<PluginConfig>,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Blacklist configuration
    pub blacklist: BlacklistConfig,
}

/// Represents configuration for a specific route.
#[derive(Debug, PartialEq)]
pub enum RouteConfig {
    /// Serve files from a directory
    Serve {
        /// Wildcard string specifying what URIs to match
        matches: String,
        /// Directory to serve files from
        directory: String,
    },
    /// Proxy connections to the specified target(s), load balancing if necessary
    Proxy {
        /// Wildcard string specifying what URIs to match
        matches: String,
        /// Load balancer instance
        load_balancer: EqMutex<LoadBalancer>,
    },
}

/// Represents configuration for the logger.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoggingConfig {
    /// The level of logging
    pub level: LogLevel,
    /// Whether to log to the console
    pub console: bool,
    /// The path to the log file
    pub file: Option<String>,
}

/// Represents configuration for the cache.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheConfig {
    /// The maximum size of the cache, in bytes
    pub size_limit: usize,
    /// The maximum time to cache an item for, in seconds
    pub time_limit: usize,
}

/// Represents configuration for the blacklist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlacklistConfig {
    /// The list of addresses to block
    pub list: Vec<String>,
    /// The way in which the blacklist is enforced
    pub mode: BlacklistMode,
}

/// Represents configuration for a plugin.
#[cfg(feature = "plugins")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginConfig {
    pub name: String,
    pub library: String,
    pub config: HashMap<String, String>,
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

impl Config {
    /// Attempts to load the configuration.
    pub fn load() -> Result<Self, String> {
        let config_string = load_config_file()
            .map_err(|_| "Could not open the specified config file or default `humphrey.conf`")?;
        let tree = parse_conf(&config_string).map_err(|e| e.to_string())?;
        let config = Self::from_tree(tree)?;

        Ok(config)
    }

    /// Parses the config from the config tree.
    pub fn from_tree(tree: ConfigNode) -> Result<Self, &'static str> {
        let mut hashmap: HashMap<String, ConfigNode> = HashMap::new();
        tree.flatten(&mut hashmap, &Vec::new());

        // Get and validate the specified address, port and threads
        let address = hashmap.get_optional("server.address", "0.0.0.0".into());
        let port: u16 = hashmap.get_optional_parsed("server.port", 80, "Invalid port")?;
        let threads: usize =
            hashmap.get_optional_parsed("server.threads", 32, "Invalid number of threads")?;
        let websocket_proxy = hashmap.get_owned("server.websocket");

        // Get and validate the blacklist file and mode
        let blacklist = {
            let blacklist = load_list_file(hashmap.get_owned("server.blacklist.file"))?;
            let blacklist_mode = hashmap.get_optional("server.blacklist.mode", "block".into());
            let blacklist_mode = match blacklist_mode.as_ref() {
                "block" => BlacklistMode::Block,
                "forbidden" => BlacklistMode::Forbidden,
                _ => return Err("Invalid blacklist mode"),
            };

            BlacklistConfig {
                list: blacklist,
                mode: blacklist_mode,
            }
        };

        // Get and validate the logging configuration
        let logging = {
            let log_level = hashmap.get_optional_parsed(
                "server.log.level",
                LogLevel::Warn,
                "Invalid log level",
            )?;
            let log_file = hashmap.get_owned("server.log.file");
            let log_console = hashmap.get_optional_parsed(
                "server.log.console",
                true,
                "server.log.console must be a boolean",
            )?;

            LoggingConfig {
                level: log_level,
                console: log_console,
                file: log_file,
            }
        };

        // Get and validate the cache configuration
        let cache = {
            let cache_size =
                hashmap.get_optional_parsed("server.cache.size", 0_usize, "Invalid cache size")?;
            let cache_time =
                hashmap.get_optional_parsed("server.cache.time", 0_usize, "Invalid cache time")?;

            CacheConfig {
                size_limit: cache_size,
                time_limit: cache_time,
            }
        };

        // Get and validate the configuration for the different routes
        let routes = {
            let routes_map = tree.get_routes();
            let mut routes: Vec<RouteConfig> = Vec::with_capacity(routes_map.len());

            for (wild, conf) in routes_map {
                if conf.contains_key("directory") {
                    // This is a regular file-serving route

                    let directory = conf.get_compulsory("directory", "").unwrap();

                    routes.push(RouteConfig::Serve {
                        matches: wild,
                        directory,
                    });
                } else if conf.contains_key("proxy") {
                    // This is a proxy route

                    let targets: Vec<String> = conf
                        .get_compulsory("proxy", "")
                        .unwrap()
                        .split(',')
                        .map(|s| s.to_owned())
                        .collect();

                    let load_balancer_mode =
                        conf.get_optional("load_balancer_mode", "round-robin".into());
                    let load_balancer_mode = match load_balancer_mode.as_str() {
                        "round-robin" => LoadBalancerMode::RoundRobin,
                        "random" => LoadBalancerMode::Random,
                        _ => return Err("Invalid load balancer mode, valid options are `round-robin` or `random`"),
                    };

                    routes.push(RouteConfig::Proxy {
                        matches: wild,
                        load_balancer: EqMutex::new(LoadBalancer {
                            targets,
                            mode: load_balancer_mode,
                            lcg: Lcg::new(),
                            index: 0,
                        }),
                    })
                } else {
                    return Err("Invalid route configuration, every route must contain either the `directory` or `proxy` field");
                }
            }

            routes
        };

        // Get and validate plugin configuration
        #[cfg(feature = "plugins")]
        let plugins = {
            let plugins_map = tree.get_plugins();
            let mut plugins: Vec<PluginConfig> = Vec::new();

            for (name, conf) in plugins_map {
                let library = conf.get_compulsory("library", "Plugin library not specified")?;
                let mut additional_config: HashMap<String, String> = conf
                    .clone()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.get_string().unwrap()))
                    .collect();
                additional_config.remove("library").unwrap();

                plugins.push(PluginConfig {
                    name,
                    library,
                    config: additional_config,
                })
            }

            plugins
        };

        Ok(Config {
            address,
            port,
            threads,
            websocket_proxy,
            routes,
            #[cfg(feature = "plugins")]
            plugins,
            logging,
            cache,
            blacklist,
        })
    }
}

/// Loads the configuration file.
fn load_config_file() -> Result<String, ()> {
    let path = args().nth(1).unwrap_or_else(|| "humphrey.conf".into());

    if let Ok(mut file) = File::open(path) {
        // The file can be opened

        let mut string = String::new();
        if file.read_to_string(&mut string).is_ok() {
            // The file can be read

            Ok(string)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

/// Loads a file which is a list of values.
fn load_list_file(path: Option<String>) -> Result<Vec<String>, &'static str> {
    if let Some(path) = path {
        // Try to open and read the file
        let mut file = File::open(path).map_err(|_| "List file could not be opened")?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|_| "List file could not be read")?;

        // Collect the lines of the file into a `Vec`
        let list: Vec<String> = buf.lines().map(|s| s.to_string()).collect();

        Ok(list)
    } else {
        // Return an empty `Vec` if no file was supplied
        Ok(Vec::new())
    }
}
