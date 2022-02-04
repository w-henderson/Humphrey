//! Provides logging functionality.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use humphrey::http::date::DateTime;
use humphrey::monitor::event::{Event, EventType, ToEventMask};

use crate::config::Config;
use crate::AppState;

/// Event mask for the `LogLevel::Error` log level.
pub const INTERNAL_MASK_ERROR: u32 = EventType::ThreadPoolPanic as u32;

/// Event mask for the `LogLevel::Warn` log level.
pub const INTERNAL_MASK_WARN: u32 = INTERNAL_MASK_ERROR
    | EventType::RequestServedError as u32
    | EventType::RequestTimeout as u32
    | EventType::StreamDisconnectedWhileWaiting as u32
    | EventType::ThreadPoolOverload as u32
    | EventType::ThreadRestarted as u32;

/// Event mask for the `LogLevel::Info` log level.
pub const INTERNAL_MASK_INFO: u32 = INTERNAL_MASK_WARN | EventType::HTTPSRedirect as u32;

/// Event mask for the `LogLevel::Debug` log level.
pub const INTERNAL_MASK_DEBUG: u32 = INTERNAL_MASK_INFO
    | EventType::KeepAliveRespected as u32
    | EventType::ThreadPoolProcessStarted as u32
    | EventType::ConnectionSuccess as u32
    | EventType::ConnectionClosed as u32;

/// Encapsulates logging methods and configuration.
pub struct Logger {
    level: LogLevel,
    console: bool,
    file: Option<Mutex<File>>,
}

/// Represents a log level.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Only errors will be logged.
    Error,
    /// Errors and warnings will be logged.
    Warn,
    /// Errors, warnings and general information will be logged.
    Info,
    /// Everything, including debug information, will be logged.
    Debug,
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

impl From<&Config> for Logger {
    fn from(config: &Config) -> Self {
        let file = config.logging.file.as_ref().map(|path| {
            Mutex::new(
                OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)
                    .unwrap(),
            )
        });

        Self {
            level: config.logging.level.clone(),
            console: config.logging.console,
            file,
        }
    }
}

impl Logger {
    /// Logs an error message.
    pub fn error(&self, message: impl AsRef<str>) {
        let string = format!("{} [ERROR] {}", Logger::time_format(), message.as_ref());
        self.log_to_console(&string);
        self.log_to_file(&string);
    }

    /// Logs a warning, provided that the log level allows this.
    pub fn warn(&self, message: impl AsRef<str>) {
        if self.level >= LogLevel::Warn {
            let string = format!("{} [WARN]  {}", Logger::time_format(), message.as_ref());
            self.log_to_console(&string);
            self.log_to_file(&string);
        }
    }

    /// Logs information, provided that the log level allows this.
    pub fn info(&self, message: impl AsRef<str>) {
        if self.level >= LogLevel::Info {
            let string = format!("{} [INFO]  {}", Logger::time_format(), message.as_ref());
            self.log_to_console(&string);
            self.log_to_file(&string);
        }
    }

    /// Logs debug information, provided that the log level allows this.
    pub fn debug(&self, message: impl AsRef<str>) {
        if self.level == LogLevel::Debug {
            let string = format!("{} [DEBUG] {}", Logger::time_format(), message.as_ref());
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

impl ToEventMask for LogLevel {
    fn to_event_mask(&self) -> u32 {
        match self {
            Self::Error => INTERNAL_MASK_ERROR,
            Self::Warn => INTERNAL_MASK_WARN,
            Self::Info => INTERNAL_MASK_INFO,
            Self::Debug => INTERNAL_MASK_DEBUG,
        }
    }
}

/// Monitors internal events and logs them.
pub fn monitor_thread(rx: Receiver<Event>, state: Arc<AppState>) {
    for e in rx {
        if e.kind == EventType::RequestServedError
            && !e
                .info
                .as_ref()
                .map(|i| i.starts_with("400"))
                .unwrap_or(false)
        {
            continue;
        }

        let message = if let Some(info) = e.info {
            if e.kind == EventType::RequestServedError || e.kind == EventType::RequestTimeout {
                format!(
                    "{}{}",
                    e.peer
                        .map(|p| p.ip().to_string() + ": ")
                        .unwrap_or_else(|| "".into()),
                    info
                )
            } else {
                format!(
                    "{}{}: {}",
                    e.peer
                        .map(|p| p.ip().to_string() + ": ")
                        .unwrap_or_else(|| "".into()),
                    e.kind.to_string(),
                    info
                )
            }
        } else {
            format!(
                "{}{}",
                e.peer
                    .map(|p| p.ip().to_string() + ": ")
                    .unwrap_or_else(|| "".into()),
                e.kind.to_string()
            )
        };

        if e.kind.to_event_mask() & INTERNAL_MASK_ERROR != 0 {
            state.logger.error(message);
        } else if e.kind.to_event_mask() & INTERNAL_MASK_WARN != 0 {
            state.logger.warn(message);
        } else if e.kind.to_event_mask() & INTERNAL_MASK_INFO != 0 {
            state.logger.info(message);
        } else {
            state.logger.debug(message);
        }
    }
}
