//! Provides the core configuration functionality.

use crate::config::extended_hashmap::ExtendedMap;
use crate::config::tree::{parse_conf, ConfigNode};
use crate::logger::LogLevel;
use crate::proxy::{EqMutex, LoadBalancer};
use crate::rand::Lcg;

use std::collections::HashMap;
use std::env::{args, var};
use std::fs::File;
use std::io::Read;
use std::net::IpAddr;
use std::path::Path;
use std::time::Duration;

/// Represents the parsed and validated configuration.
#[derive(Debug, PartialEq)]
pub struct Config {
    /// Where the configuration was located.
    pub source: ConfigSource,
    /// The address to host the server on
    pub address: String,
    /// The port to host the server on
    pub port: u16,
    /// The number of threads to host the server on
    pub threads: usize,
    /// The TLS configuration to use
    #[cfg(feature = "tls")]
    pub tls_config: Option<TlsConfig>,
    /// Address to forward WebSocket connections to, unless otherwise specified by the route
    pub default_websocket_proxy: Option<String>,
    /// The configuration for different hosts
    pub hosts: Vec<HostConfig>,
    /// The configuration for the default host
    pub default_host: HostConfig,
    /// The configuration for any plugins
    #[cfg(feature = "plugins")]
    pub plugins: Vec<PluginConfig>,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Blacklist configuration
    pub blacklist: BlacklistConfig,
    /// The amount of time to wait between requests
    pub connection_timeout: Option<Duration>,
}

/// Represents the configuration for a specific host.
#[derive(Debug, PartialEq)]
pub struct HostConfig {
    /// Wildcard string specifying what hosts to match, e.g. `*.example.com`
    pub matches: String,
    /// The routes to use for this host
    pub routes: Vec<RouteConfig>,
}

/// Represents the type of a route.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RouteType {
    /// Serve a single file.
    File,
    /// Serve a directory of files.
    Directory,
    /// Proxy requests to this route to another server.
    Proxy,
    /// Redirect clients to another server.
    Redirect,
    /// Proxies WebSocket requests to this route to another server.
    ExclusiveWebSocket,
}

/// Represents configuration for a specific route.
#[derive(Debug, PartialEq)]
pub struct RouteConfig {
    /// The type of the route
    pub route_type: RouteType,
    /// The URL to match
    pub matches: String,
    /// The path to the file, directory or redirect target
    pub path: Option<String>,
    /// The load balancer to use for proxying
    pub load_balancer: Option<EqMutex<LoadBalancer>>,
    /// The WebSocket proxy target for WebSocket connections to this route
    pub websocket_proxy: Option<String>,
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
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
    pub list: Vec<IpAddr>,
    /// The way in which the blacklist is enforced
    pub mode: BlacklistMode,
}

/// Represents configuration for TLS.
#[cfg(feature = "tls")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TlsConfig {
    /// The TLS certificate file path.
    pub cert_file: String,
    /// The TLS key file path.
    pub key_file: String,
    /// Whether to force clients to use HTTPS.
    pub force: bool,
}

/// Represents configuration for a plugin.
#[cfg(feature = "plugins")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginConfig {
    /// The name of the plugin.
    pub name: String,
    /// The path to the shared library file.
    pub library: String,
    /// The configuration for the plugin.
    pub config: HashMap<String, String>,
}

/// Represents an algorithm for load balancing.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LoadBalancerMode {
    /// Evenly distributes load in a repeating pattern
    RoundRobin,
    /// Randomly distributes load
    Random,
}

/// Represents a method of applying the blacklist.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlacklistMode {
    /// Does not allow any access from blacklisted addresses
    Block,
    /// Returns 400 Forbidden to every request, only available in static mode
    Forbidden,
}

/// Represents the source of the configuration.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConfigSource {
    /// The configuration was found at a path specified in a command-line argument.
    Argument,
    /// The configuration was found at a path specified in the environment variable `HUMPHREY_CONF`.
    EnvironmentVariable,
    /// The configuration was found in the current directory at `humphrey.conf`.
    CurrentDirectory,
    /// The configuration was not found, so the default configuration was used.
    Default,
}

impl Config {
    /// Attempts to load the configuration.
    pub fn load() -> Result<Self, String> {
        let (path, source) = if let Some(arg_path) = args().nth(1) {
            (arg_path, ConfigSource::Argument)
        } else if Path::new("humphrey.conf").exists() {
            ("humphrey.conf".into(), ConfigSource::CurrentDirectory)
        } else if let Ok(env_path) = var("HUMPHREY_CONF") {
            (env_path, ConfigSource::EnvironmentVariable)
        } else {
            ("".into(), ConfigSource::Default)
        };

        if let Ok((filename, config_string)) = load_config_file(path) {
            let tree = parse_conf(&config_string, &filename).map_err(|e| e.to_string())?;
            let mut config = Self::from_tree(tree)?;
            config.source = source;

            Ok(config)
        } else {
            Ok(Config::default())
        }
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
        let default_websocket_proxy = hashmap.get_owned("server.websocket");
        let connection_timeout_seconds: u64 =
            hashmap.get_optional_parsed("server.timeout", 0, "Invalid connection timeout")?;
        let connection_timeout = if connection_timeout_seconds > 0 {
            Some(Duration::from_secs(connection_timeout_seconds))
        } else {
            None
        };

        if threads < 1 {
            return Err("You cannot specify less than 1 thread");
        }

        // Get and validate the blacklist file and mode
        let blacklist = {
            let blacklist_strings: Vec<String> =
                load_list_file(hashmap.get_owned("server.blacklist.file"))?;
            let mut blacklist: Vec<IpAddr> = Vec::with_capacity(blacklist_strings.len());

            for ip in blacklist_strings {
                blacklist.push(
                    ip.parse::<IpAddr>()
                        .map_err(|_| "Could not parse IP address in blacklist file")?,
                );
            }

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

        #[cfg(feature = "tls")]
        let tls_config = {
            let cert_file = hashmap.get_owned("server.tls.cert_file");
            let key_file = hashmap.get_owned("server.tls.key_file");
            let force = hashmap.get_optional("server.tls.force", "false".into());

            if force == "true" && threads < 2 {
                return Err("A minimum of two threads are required to force HTTPS");
            }

            if force == "true" && port != 443 {
                return Err("Forcing HTTPS redirects requires the port to be 443");
            }

            if let Some(cert_file) = cert_file {
                if let Some(key_file) = key_file {
                    Some(TlsConfig {
                        cert_file,
                        key_file,
                        force: force == "true",
                    })
                } else {
                    return Err("Missing key file for TLS");
                }
            } else {
                None
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
        let default_host = parse_host("*", &tree)?;

        let hosts = {
            let hosts_map = tree.get_hosts();
            let mut hosts: Vec<HostConfig> = Vec::with_capacity(hosts_map.len());

            for (host, conf) in hosts_map {
                hosts.push(parse_host(&host, &conf)?);
            }

            hosts
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
            source: ConfigSource::Default,
            address,
            port,
            threads,
            #[cfg(feature = "tls")]
            tls_config,
            default_websocket_proxy,
            default_host,
            hosts,
            #[cfg(feature = "plugins")]
            plugins,
            logging,
            cache,
            blacklist,
            connection_timeout,
        })
    }

    /// Get the route at the given host and route indices.
    pub fn get_route(&self, host: usize, route: usize) -> &RouteConfig {
        if host == 0 {
            &self.default_host.routes[route]
        } else {
            &self.hosts[host - 1].routes[route]
        }
    }
}

/// Loads the configuration file.
fn load_config_file(path: impl AsRef<str>) -> Result<(String, String), ()> {
    if let Ok(mut file) = File::open(path.as_ref()) {
        // The file can be opened

        let mut string = String::new();
        if file.read_to_string(&mut string).is_ok() {
            // The file can be read

            Ok((path.as_ref().to_string(), string))
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

/// Parses a node which contains the configuration for a host.
fn parse_host(wild: &str, node: &ConfigNode) -> Result<HostConfig, &'static str> {
    let routes_map = node.get_routes();
    let mut routes: Vec<RouteConfig> = Vec::with_capacity(routes_map.len());

    for (wild, conf) in routes_map {
        routes.extend(parse_route(&wild, conf)?);
    }

    Ok(HostConfig {
        matches: wild.to_string(),
        routes,
    })
}

/// Parses a route.
fn parse_route(
    wild: &str,
    conf: HashMap<String, ConfigNode>,
) -> Result<Vec<RouteConfig>, &'static str> {
    let mut routes: Vec<RouteConfig> = Vec::new();

    for wild in wild.split(',').map(|s| s.trim()) {
        let websocket_proxy = conf.get_owned("websocket");

        if conf.contains_key("file") {
            // This is a regular file-serving route

            let file = conf.get_compulsory("file", "").unwrap();

            routes.push(RouteConfig {
                route_type: RouteType::File,
                matches: wild.to_string(),
                path: Some(file),
                load_balancer: None,
                websocket_proxy,
            });
        } else if conf.contains_key("directory") {
            // This is a regular directory-serving route

            let directory = conf.get_compulsory("directory", "").unwrap();

            routes.push(RouteConfig {
                route_type: RouteType::Directory,
                matches: wild.to_string(),
                path: Some(directory),
                load_balancer: None,
                websocket_proxy,
            });
        } else if conf.contains_key("proxy") {
            // This is a proxy route

            let targets: Vec<String> = conf
                .get_compulsory("proxy", "")
                .unwrap()
                .split(',')
                .map(|s| s.to_owned())
                .collect();

            let load_balancer_mode = conf.get_optional("load_balancer_mode", "round-robin".into());
            let load_balancer_mode =
                match load_balancer_mode.as_str() {
                    "round-robin" => LoadBalancerMode::RoundRobin,
                    "random" => LoadBalancerMode::Random,
                    _ => return Err(
                        "Invalid load balancer mode, valid options are `round-robin` or `random`",
                    ),
                };

            let load_balancer = EqMutex::new(LoadBalancer {
                targets,
                mode: load_balancer_mode,
                lcg: Lcg::new(),
                index: 0,
            });

            routes.push(RouteConfig {
                route_type: RouteType::Proxy,
                matches: wild.to_string(),
                path: None,
                load_balancer: Some(load_balancer),
                websocket_proxy,
            });
        } else if conf.contains_key("redirect") {
            // This is a redirect route

            let target = conf.get_compulsory("redirect", "").unwrap();

            routes.push(RouteConfig {
                route_type: RouteType::Redirect,
                matches: wild.to_string(),
                path: Some(target),
                load_balancer: None,
                websocket_proxy,
            });
        } else if !conf.contains_key("websocket") {
            return Err("Invalid route configuration, every route must contain either the `file`, `directory`, `proxy` or `redirect` field, unless it defines a WebSocket proxy with the `websocket` field");
        } else {
            routes.push(RouteConfig {
                route_type: RouteType::ExclusiveWebSocket,
                matches: wild.to_string(),
                path: None,
                load_balancer: None,
                websocket_proxy,
            });
        }
    }

    Ok(routes)
}
