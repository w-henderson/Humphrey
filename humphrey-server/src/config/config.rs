use crate::config::tree::ConfigNode;
use crate::logger::LogLevel;

use std::collections::HashMap;

/// Represents the parsed and validated configuration.
pub struct Config {
    /// The address to host the server on
    address: String,
    /// The port to host the server on
    port: u16,
    /// The number of threads to host the server on
    threads: usize,
    /// The configuration for different routes
    routes: Vec<RouteConfig>,
    /// The configuration for any plugins
    #[cfg(feature = "plugins")]
    plugins: Vec<PluginConfig>,
    /// Logging configuration
    logging: LoggingConfig,
    /// Cache configuration
    cache: Option<CacheConfig>,
    /// Blacklist configuration
    blacklist: Option<BlacklistConfig>,
}

/// Represents configuration for a specific route.
pub enum RouteConfig {
    /// Serve files from a directory
    Serve {
        /// Wildcard string specifying what URIs to match
        matches: String,
        /// Directory to serve files from
        directory: String,
        /// Address to forward WebSocket connections to
        websocket_proxy: Option<String>,
    },
    /// Proxy connections to the specified target(s), load balancing if necessary
    Proxy {
        /// Wildcard string specifying what URIs to match
        matches: String,
        /// Proxy targets
        targets: Vec<String>,
        /// Algorithm for load balancing
        load_balancer_mode: LoadBalancerMode,
    },
}

/// Represents configuration for the logger.
pub struct LoggingConfig {
    /// The level of logging
    level: LogLevel,
    /// Whether to log to the console
    console: bool,
    /// The path to the log file
    file: Option<String>,
}

/// Represents configuration for the cache.
pub struct CacheConfig {
    /// The maximum size of the cache, in bytes
    size_limit: usize,
    /// The maximum time to cache an item for, in seconds
    time_limit: usize,
}

/// Represents configuration for the blacklist.
pub struct BlacklistConfig {
    /// The list of addresses to block
    list: Vec<String>,
    /// The way in which the blacklist is enforced
    mode: BlacklistMode,
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
    pub fn from_tree(tree: ConfigNode) -> Result<Self, &'static str> {
        let mut hashmap: HashMap<String, ConfigNode> = HashMap::new();
        tree.flatten(&mut hashmap, &Vec::new());

        Err("sadge")

        /*let address = tree
        .get_children()
        .unwrap()
        .iter()
        .find(|n| &n.get_key() == "address")?
        .get_string()?;*/
    }
}
