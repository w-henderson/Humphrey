//! Provides functionality for working with the configuration syntax tree.

use crate::config::error::ConfigError;
use crate::config::traceback::TracebackIterator;
use humphrey::krauss::wildcard_match;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::str::Lines;

/// Represents a node in the configuration syntax tree.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ConfigNode {
    /// A node that contains a number.
    Number(String, String),
    /// A node that contains a boolean.
    Boolean(String, String),
    /// A node that contains a string.
    String(String, String),
    /// A node that represents a section and contains a number of child nodes.
    Section(String, Vec<ConfigNode>),
    /// A node that represents a host's configuration and contains a number of child nodes.
    Host(String, Vec<ConfigNode>),
    /// A node that represents a route's configuration and contains a number of child nodes.
    Route(String, Vec<ConfigNode>),
}

impl ConfigNode {
    /// Flatten this node and all child nodes into a hashmap.
    pub fn flatten(&self, hashmap: &mut HashMap<String, Self>, level: &[&str]) {
        match self {
            ConfigNode::Section(k, v) => {
                if k != "plugins" {
                    let mut new_level = level.to_vec();
                    new_level.push(k);
                    for child in v {
                        child.flatten(hashmap, &new_level);
                    }
                }
            }
            ConfigNode::Number(k, _) | ConfigNode::Boolean(k, _) | ConfigNode::String(k, _) => {
                let mut new_level = level.to_vec();
                new_level.push(k);
                hashmap.insert(new_level.join("."), self.clone());
            }
            _ => (),
        }
    }

    /// Get the hosts configured under this node.
    pub fn get_hosts(&self) -> Vec<(String, Self)> {
        let mut hosts: Vec<(String, Self)> = Vec::new();

        if let ConfigNode::Section(_, children) = self {
            for child in children {
                if let ConfigNode::Host(wild, _) = child {
                    hosts.push((wild.clone(), child.clone()));
                }
            }
        }

        hosts
    }

    /// Get the routes configured under this node.
    pub fn get_routes(&self) -> Vec<(String, HashMap<String, Self>)> {
        let mut routes: Vec<(String, HashMap<String, Self>)> = Vec::new();

        if let ConfigNode::Section(_, children) = self {
            for child in children {
                if let ConfigNode::Route(wild, inner_children) = child {
                    let mut inner_hashmap: HashMap<String, Self> = HashMap::new();
                    for inner_child in inner_children {
                        inner_child.flatten(&mut inner_hashmap, &Vec::new());
                    }

                    routes.push((wild.clone(), inner_hashmap));
                }
            }
        }

        if let ConfigNode::Host(_, children) = self {
            for child in children {
                if let ConfigNode::Route(wild, inner_children) = child {
                    let mut inner_hashmap: HashMap<String, Self> = HashMap::new();
                    for inner_child in inner_children {
                        inner_child.flatten(&mut inner_hashmap, &Vec::new());
                    }

                    routes.push((wild.clone(), inner_hashmap));
                }
            }
        }

        routes
    }

    /// Get the plugins configured under this node.
    pub fn get_plugins(&self) -> Vec<(String, HashMap<String, Self>)> {
        let mut plugins: Vec<(String, HashMap<String, Self>)> = Vec::new();
        if let ConfigNode::Section(_, children) = self {
            for child in children {
                if let ConfigNode::Section(name, inner_children) = child {
                    if name == "plugins" {
                        for inner_child in inner_children {
                            if let ConfigNode::Section(inner_name, inner_inner_children) =
                                inner_child
                            {
                                let mut inner_hashmap: HashMap<String, Self> = HashMap::new();
                                for inner_inner_child in inner_inner_children {
                                    inner_inner_child.flatten(&mut inner_hashmap, &Vec::new());
                                }

                                plugins.push((inner_name.clone(), inner_hashmap));
                            }
                        }
                    }
                }
            }
        }

        plugins
    }

    /// Get this node's value as a string, or `None` if this is not possible.
    pub fn get_string(&self) -> Option<String> {
        match self {
            ConfigNode::String(_, s) => Some(s.clone()),
            ConfigNode::Number(_, n) => Some(n.clone()),
            ConfigNode::Boolean(_, b) => Some(b.clone()),
            _ => None,
        }
    }
}

/// Parses an entire configuration string.
pub fn parse_conf(conf: &str, filename: &str) -> Result<ConfigNode, ConfigError> {
    let mut lines = TracebackIterator::from(conf.lines());

    // Attemps to find the start of the configuration
    let mut line_content = "";
    while line_content != "server {" {
        if let Some(line) = lines.next() {
            line_content = clean_up(line);
        } else {
            return Err(ConfigError::new(
                "Could not find `server` section",
                filename,
                0,
            ));
        }
    }

    // Parses the main section
    parse_section("server", &mut lines, filename)
}

/// Recursively parses a section of the configuration.
fn parse_section(
    name: &str,
    lines: &mut TracebackIterator<Lines>,
    filename: &str,
) -> Result<ConfigNode, ConfigError> {
    let mut values: Vec<ConfigNode> = Vec::new();

    // While this section has not ended
    loop {
        // Attempt to read a line

        if let Some(line) = lines.next() {
            let line = clean_up(line);

            if let Some(section_name) = line.strip_suffix('{') {
                // If the line indicates the start of a section, recursively parse that section

                let section_name = section_name.trim();
                if section_name.starts_with("route ") && section_name != "route {" {
                    // If the section is a route section, parse it as such
                    let route_name = section_name.splitn(2, ' ').last().unwrap().trim();
                    let section = parse_section(route_name, lines, filename)?;
                    if let ConfigNode::Section(route_name, inner_values) = section {
                        values.push(ConfigNode::Route(route_name, inner_values));
                    }
                } else if section_name.starts_with("host ") && section_name != "host {" {
                    // If the section is a host section, parse it as such
                    let host_name = {
                        let raw = section_name.splitn(2, ' ').last().unwrap().trim();
                        if raw.starts_with('\"') && raw.ends_with('\"') {
                            raw[1..raw.len() - 1].to_string()
                        } else {
                            raw.to_string()
                        }
                    };

                    let section = parse_section(&host_name, lines, filename)?;
                    if let ConfigNode::Section(host_name, inner_values) = section {
                        values.push(ConfigNode::Host(host_name, inner_values));
                    }
                } else {
                    // If the section is just a regular section, parse it in the normal way
                    values.push(parse_section(section_name, lines, filename)?);
                }
            } else if line == "}" {
                // If the line indicates the end of this section, return the parsed section

                break;
            } else if !line.is_empty() {
                // If the line is not empty, attempt to parse the value

                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                quiet_assert(parts.len() == 2, "Syntax error", filename, lines)?;

                let key = parts[0].trim();
                let value = parts[1].trim();

                // If this is just a regular value
                if key != "include" {
                    if wildcard_match("\"*\"", value) {
                        values.push(ConfigNode::String(
                            key.into(),
                            value[1..value.len() - 1].into(),
                        ))
                    } else if value.parse::<i64>().is_ok() {
                        values.push(ConfigNode::Number(key.into(), value.into()))
                    } else if value.parse::<bool>().is_ok() {
                        values.push(ConfigNode::Boolean(key.into(), value.into()))
                    } else if let Ok(size) = parse_size(value) {
                        values.push(ConfigNode::Number(key.into(), size.to_string()))
                    } else {
                        return Err(ConfigError::new(
                            "Could not parse value",
                            filename,
                            lines.current_line(),
                        ));
                    }
                } else if wildcard_match("\"*\"", value) {
                    let include_result =
                        include(&value[1..value.len() - 1], filename, lines.current_line());
                    if let Ok(included_nodes) = include_result {
                        values.extend(included_nodes);
                    } else {
                        return Err(include_result.unwrap_err());
                    }
                } else {
                    return Err(ConfigError::new(
                        "Invalid include value, it takes a file path in quotation marks as its value",
                        filename,
                        lines.current_line(),
                    ));
                }
            }
        } else {
            // If the line could not be read, return an error

            return Err(ConfigError::new(
                "Unexpected end of file, expected `}`",
                filename,
                lines.current_line(),
            ));
        }
    }

    Ok(ConfigNode::Section(name.into(), values))
}

/// Attempts to include the configuration file at the specified path into the tree,
///   returning a `Vec` of `ConfigNode`s. If unsuccessful, returns a descriptive error.
fn include(path: &str, containing_file: &str, line: u64) -> Result<Vec<ConfigNode>, ConfigError> {
    if let Ok(mut file) = File::open(path) {
        let mut buf = String::new();
        if file.read_to_string(&mut buf).is_ok() {
            buf.push_str("\n}");

            let mut iter = TracebackIterator::from(buf.lines());
            let parsed_node = parse_section("temp_included_section", &mut iter, path)?;

            match parsed_node {
                ConfigNode::Section(_, children) => Ok(children),
                _ => Err(ConfigError::new(
                    "Internal parser error",
                    containing_file,
                    line,
                )),
            }
        } else {
            Err(ConfigError::new(
                "Could not read included file",
                containing_file,
                line,
            ))
        }
    } else {
        Err(ConfigError::new(
            "Could not open included file",
            containing_file,
            line,
        ))
    }
}

/// Cleans up a line by removing comments and trailing whitespace.
fn clean_up(line: &str) -> &str {
    line.split_once('#').map_or(line, |x| x.0).trim()
}

/// Parses a size string into its corresponding number of bytes.
/// For example, 4K => 4096, 1M => 1048576.
/// If no letter is provided at the end, assumes the number to be in bytes.
fn parse_size(size: &str) -> Result<i64, ()> {
    if size.is_empty() {
        // Empty string

        Err(())
    } else if size.len() == 1 {
        // One character so cannot possibly be valid

        size.parse::<i64>().map_err(|_| ())
    } else {
        let last_char = size.chars().last().unwrap().to_ascii_uppercase();
        let number: i64 = size[0..size.len() - 1].parse().map_err(|_| ())?;

        match last_char {
            'K' => Ok(number * 1024),
            'M' => Ok(number * 1024 * 1024),
            'G' => Ok(number * 1024 * 1024 * 1024),
            '0'..='9' => size.parse::<i64>().map_err(|_| ()),
            _ => Err(()),
        }
    }
}

/// Asserts a condition, returning a `Result` rather than panicking like the `assert!` macro.
fn quiet_assert<T>(
    condition: bool,
    message: &'static str,
    filename: &str,
    iter: &mut TracebackIterator<T>,
) -> Result<(), ConfigError>
where
    T: Iterator,
{
    match condition {
        true => Ok(()),
        false => Err(ConfigError::new(message, filename, iter.current_line())),
    }
}
