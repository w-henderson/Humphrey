//! Event types for monitoring.

use crate::http::date::DateTime;

use std::borrow::Cow;
use std::fmt::Display;
use std::net::{SocketAddr, ToSocketAddrs};

/// Represents a monitoring event.
pub struct Event {
    /// The type of the event.
    pub kind: EventType,
    /// The address of the peer that triggered the event, if applicable.
    pub peer: Option<SocketAddr>,
    /// Additional information about the event, if applicable.
    pub info: Option<Cow<'static, str>>,
}

/// Represents the type of event.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventType {
    /// A successful connection.
    ConnectionSuccess = 0x01,
    /// The connection was denied.
    ConnectionDenied = 0x02,
    /// An error occurred while connecting.
    ConnectionError = 0x04,
    /// The connection was closed.
    ConnectionClosed = 0x08,
    /// The process started running in the thread pool.
    ThreadPoolProcessStarted = 0x10,
    /// The stream disconnected while waiting to be processed.
    StreamDisconnectedWhileWaiting = 0x20,
    /// A request was served successfully.
    RequestServedSuccess = 0x40,
    /// A request was served, but an error was encountered.
    RequestServedError = 0x80,
    /// A request timed out.
    RequestTimeout = 0x0100,
    /// A connection was held open due to the `Keep-Alive` header.
    KeepAliveRespected = 0x0200,
    /// A WebSocket connection was requested.
    WebsocketConnectionRequested = 0x0400,
    /// A WebSocket connection was closed.
    WebsocketConnectionClosed = 0x0800,
    /// A client was redirected to use HTTPS.
    HTTPSRedirect = 0x1000,
    /// The thread pool is overloaded.
    ThreadPoolOverload = 0x2000,
    /// The thread pool recovered from a thread error.
    ThreadPoolPanic = 0x4000,
}

/// Represents a category of events.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLevel {
    /// Only critical errors are logged.
    Error = 0b100_0000_1000_0100,
    /// Only errors and warnings are logged.
    Warning = 0b110_0001_1010_0110,
    /// Informative messages are logged.
    Info = 0b111_1101_1110_1110,
    /// Everything is logged.
    Debug = u32::MAX,
}

/// Represents a type which can be converted to an event mask.
pub trait ToEventMask {
    /// Convert to an event mask.
    fn to_event_mask(&self) -> u32;
}

impl Event {
    /// Create a new event with the given event type.
    pub fn new(kind: EventType) -> Self {
        Self {
            kind,
            peer: None,
            info: None,
        }
    }

    /// Add a peer to the event.
    pub fn with_peer(mut self, peer: impl ToSocketAddrs) -> Self {
        self.peer = Some(peer.to_socket_addrs().unwrap().next().unwrap());
        self
    }

    /// Add a peer to the event, if the result is `Ok`.
    pub fn with_peer_result<T, E>(mut self, peer_result: Result<T, E>) -> Self
    where
        T: ToSocketAddrs,
        E: std::fmt::Display,
    {
        if let Ok(peer) = peer_result {
            if let Ok(mut peer) = peer.to_socket_addrs() {
                self.peer = peer.next();
            }
        }

        self
    }

    /// Adds information to the event.
    pub fn with_info<T>(mut self, info: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.info = Some(info.into());
        self
    }
}

impl ToString for EventType {
    fn to_string(&self) -> String {
        let string: &'static str = (*self).into();
        string.to_string()
    }
}

impl From<EventType> for &'static str {
    fn from(kind: EventType) -> Self {
        match kind {
            EventType::ConnectionSuccess => "Connection successful",
            EventType::ConnectionDenied => "Connection denied",
            EventType::ConnectionError => "Connection error",
            EventType::ConnectionClosed => "Connection closed",
            EventType::ThreadPoolProcessStarted => "Handler sent in thread pool",
            EventType::StreamDisconnectedWhileWaiting => {
                "Stream disconnected while waiting to be processed"
            }
            EventType::RequestServedSuccess => "Request served",
            EventType::RequestServedError => "Request error",
            EventType::RequestTimeout => "Request timeout",
            EventType::KeepAliveRespected => "Connection kept alive after response",
            EventType::WebsocketConnectionRequested => "WebSocket connection requested",
            EventType::WebsocketConnectionClosed => "WebSocket connection closed",
            EventType::HTTPSRedirect => "Redirected to HTTPS",
            EventType::ThreadPoolOverload => "Thread pool overloaded",
            EventType::ThreadPoolPanic => "Thread pool panic",
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time = DateTime::now();

        write!(
            f,
            "{}-{:02}-{:02} {:02}:{:02}:{:02} ",
            time.year,
            time.month + 1,
            time.day,
            time.hour,
            time.minute,
            time.second
        )?;

        if let Some(info) = self.info.as_ref() {
            write!(
                f,
                "{}{}: {}",
                self.peer
                    .map(|p| p.to_string() + " ")
                    .unwrap_or_else(|| "".into()),
                self.kind.to_string(),
                info
            )
        } else {
            write!(
                f,
                "{}{}",
                self.peer
                    .map(|p| p.to_string() + " ")
                    .unwrap_or_else(|| "".into()),
                self.kind.to_string()
            )
        }
    }
}

impl From<EventType> for Event {
    fn from(kind: EventType) -> Self {
        Self::new(kind)
    }
}

impl ToEventMask for EventType {
    fn to_event_mask(&self) -> u32 {
        *self as u32
    }
}

impl ToEventMask for EventLevel {
    fn to_event_mask(&self) -> u32 {
        *self as u32
    }
}

impl ToEventMask for u32 {
    fn to_event_mask(&self) -> u32 {
        *self
    }
}
