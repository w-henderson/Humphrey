use crate::config::error::ConfigError;
use crate::config::traceback::TracebackIterator;
use humphrey::krauss::wildcard_match;

use std::str::Lines;

/// Represents a configuration value.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ConfigValue {
    Number(String, i64),
    Boolean(String, bool),
    String(String, String),
    FilePath(String, String),
    Section(String, Vec<ConfigValue>),
    Route(String, Vec<ConfigValue>),
}

/// Parses an entire configuration string.
pub fn parse_conf(conf: &str) -> Result<ConfigValue, ConfigError> {
    let mut lines = TracebackIterator::from(conf.lines());

    // Attemps to find the start of the configuration
    let mut line_content = "";
    while line_content != "server {" {
        if let Some(line) = lines.next() {
            line_content = clean_up(line);
        } else {
            return Err(ConfigError::new("Could not find `server` section", 0));
        }
    }

    // Parses the main section
    parse_section("server", &mut lines)
}

/// Recursively parses a section of the configuration.
fn parse_section(
    name: &str,
    lines: &mut TracebackIterator<Lines>,
) -> Result<ConfigValue, ConfigError> {
    let mut section_open: bool = true;
    let mut values: Vec<ConfigValue> = Vec::new();

    // While this section has not been closed
    while section_open {
        // Attempt to read a line

        if let Some(line) = lines.next() {
            let line = clean_up(line);

            if line.ends_with('{') {
                // If the line indicates the start of a section, recursively parse that section

                let section_name = line[..line.len() - 1].trim();
                if section_name.starts_with("route ") && section_name != "route {" {
                    // If the section is a route section, parse it as such
                    let route_name = section_name.splitn(2, ' ').last().unwrap().trim();
                    let section = parse_section(route_name, lines)?;
                    if let ConfigValue::Section(route_name, inner_values) = section {
                        values.push(ConfigValue::Route(route_name, inner_values));
                    }
                } else {
                    // If the section is just a regular section, parse it in the normal way
                    values.push(parse_section(section_name, lines)?);
                }
            } else if line == "}" {
                // If the line indicates the end of this section, return the parsed section

                section_open = false;
            } else if line != "" {
                // If the line is not empty, attempt to parse the value

                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                quiet_assert(parts.len() == 2, "Syntax error", lines)?;

                let key = parts[0].trim();
                let value = parts[1].trim();

                if wildcard_match("\"*\"", value) {
                    if key == "file" {
                        values.push(ConfigValue::FilePath(
                            key.into(),
                            value[1..value.len() - 1].into(),
                        ))
                    } else {
                        values.push(ConfigValue::String(
                            key.into(),
                            value[1..value.len() - 1].into(),
                        ))
                    }
                } else if let Ok(number) = value.parse::<i64>() {
                    values.push(ConfigValue::Number(key.into(), number))
                } else if let Ok(boolean) = value.parse::<bool>() {
                    values.push(ConfigValue::Boolean(key.into(), boolean))
                } else if let Ok(size) = parse_size(value) {
                    values.push(ConfigValue::Number(key.into(), size))
                } else {
                    return Err(ConfigError::new(
                        "Could not parse value",
                        lines.current_line(),
                    ));
                }
            }
        } else {
            // If the line could not be read, return an error

            return Err(ConfigError::new(
                "Unexpected end of file, expected `}`",
                lines.current_line(),
            ));
        }
    }

    Ok(ConfigValue::Section(name.into(), values))
}

/// Cleans up a line by removing comments and trailing whitespace.
fn clean_up(line: &str) -> &str {
    line.splitn(2, "#").next().unwrap().trim()
}

/// Parses a size string into its corresponding number of bytes.
/// For example, 4K => 4096, 1M => 1048576.
/// If no letter is provided at the end, assumes the number to be in bytes.
fn parse_size(size: &str) -> Result<i64, ()> {
    if size.len() == 0 {
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
    iter: &mut TracebackIterator<T>,
) -> Result<(), ConfigError>
where
    T: Iterator,
{
    match condition {
        true => Ok(()),
        false => Err(ConfigError::new(message, iter.current_line())),
    }
}
