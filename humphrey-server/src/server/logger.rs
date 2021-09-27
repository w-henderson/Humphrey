use std::fs::{File, OpenOptions};
use std::io::Write;
use std::str::FromStr;
use std::sync::Mutex;

use crate::config::Config;
use humphrey::http::date::DateTime;

/// Encapsulates logging methods and configuration.
pub struct Logger {
    level: LogLevel,
    console: bool,
    file: Option<Mutex<File>>,
}

impl From<&Config> for Logger {
    fn from(config: &Config) -> Self {
        let file = if let Some(path) = &config.logging.file {
            // If the log file can be opened, wrap it in a `Mutex`
            Some(Mutex::new(
                OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)
                    .unwrap(),
            ))
        } else {
            // Otherwise don't log to a file
            None
        };

        Self {
            level: config.logging.level.clone(),
            console: config.logging.console,
            file,
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            level: LogLevel::Warn,
            console: true,
            file: None,
        }
    }
}

impl Logger {
    /// Logs an error message.
    pub fn error(&self, message: &str) {
        let string = format!("{} [ERROR] {}", Logger::time_format(), message);
        self.log_to_console(&string);
        self.log_to_file(&string);
    }

    /// Logs a warning, provided that the log level allows this.
    pub fn warn(&self, message: &str) {
        if self.level >= LogLevel::Warn {
            let string = format!("{} [WARN]  {}", Logger::time_format(), message);
            self.log_to_console(&string);
            self.log_to_file(&string);
        }
    }

    /// Logs information, provided that the log level allows this.
    pub fn info(&self, message: &str) {
        if self.level >= LogLevel::Info {
            let string = format!("{} [INFO]  {}", Logger::time_format(), message);
            self.log_to_console(&string);
            self.log_to_file(&string);
        }
    }

    /// Logs debug information, provided that the log level allows this.
    pub fn debug(&self, message: &str) {
        if self.level == LogLevel::Debug {
            let string = format!("{} [DEBUG] {}", Logger::time_format(), message);
            self.log_to_console(&string);
            self.log_to_file(&string);
        }
    }

    /// Formats the current time into the format `YYYY-MM-DD HH:MM:SS`
    fn time_format() -> String {
        let time = DateTime::now();
        format!(
            "{}-{:02}-{:02} {:02}:{:02}:{:02}",
            time.year,
            time.month + 1,
            time.day,
            time.hour,
            time.minute,
            time.second
        )
    }

    /// Logs the string to the console, if the logging configuration allows it
    fn log_to_console(&self, string: &str) {
        if self.console {
            println!("{}", string);
        }
    }

    /// Logs the string to the log file, if the logging configuration allows it
    fn log_to_file(&self, string: &str) {
        if let Some(file_mutex) = &self.file {
            let mut file = file_mutex.lock().unwrap();
            file.write_all(string.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
        }
    }
}

/// Represents a log level.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Only errors will be logged
    Error,
    /// Errors and warnings will be logged
    Warn,
    /// Errors, warnings and general information will be logged
    Info,
    /// Everything, including debug information, will be logged
    Debug,
}

impl FromStr for LogLevel {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            _ => Err("Log level was invalid"),
        }
    }
}
