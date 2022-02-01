use crate::http::date::DateTime;

use std::borrow::Cow;
use std::fmt::Display;
use std::net::{SocketAddr, ToSocketAddrs};

pub struct Event {
    pub kind: EventType,
    pub peer: Option<SocketAddr>,
    pub info: Option<Cow<'static, str>>,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventType {
    ConnectionSuccess = 0x01,
    ConnectionDenied = 0x02,
    ConnectionError = 0x04,
    ConnectionClosed = 0x08,
    ThreadPoolProcessStarted = 0x10,
    StreamDisconnectedWhileWaiting = 0x20,
    RequestServedSuccess = 0x40,
    RequestServedError = 0x80,
    RequestTimeout = 0x0100,
    KeepAliveRespected = 0x0200,
    WebsocketConnectionRequested = 0x0400,
    WebsocketConnectionClosed = 0x0800,
    HTTPSRedirect = 0x1000,
    ThreadPoolOverload = 0x2000,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLevel {
    Error = 0b00_0000_1000_0100,
    Warning = 0b10_0001_1010_0110,
    Info = 0b11_1101_1110_1110,
    Debug = u32::MAX,
}

pub trait ToEventMask {
    fn to_event_mask(&self) -> u32;
}

impl Event {
    pub fn new(kind: EventType) -> Self {
        Self {
            kind,
            peer: None,
            info: None,
        }
    }

    pub fn with_peer(mut self, peer: impl ToSocketAddrs) -> Self {
        self.peer = Some(peer.to_socket_addrs().unwrap().next().unwrap());
        self
    }

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
