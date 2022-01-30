use std::borrow::Cow;
use std::net::{SocketAddr, ToSocketAddrs};

pub struct Event {
    pub kind: EventType,
    pub peer: Option<SocketAddr>,
    pub info: Option<Cow<'static, str>>,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventType {
    ConnectionAttempt = 0x01,
    ConnectionSuccess = 0x02,
    ConnectionDenied = 0x04,
    ConnectionFailed = 0x08,
    ConnectionClosed = 0x10,
    ThreadPoolProcessStarted = 0x20,
    StreamDisconnectedWhileWaiting = 0x40,
    RequestAttempt = 0x80,
    RequestSuccess = 0x0100,
    RequestFailed = 0x0200,
    ResponseAttempt = 0x0400,
    ResponseSuccess = 0x0800,
    ResponseFailed = 0x1000,
    KeepAliveRespected = 0x2000,
    WebsocketConnectionRequested = 0x4000,
    WebsocketConnectionClosed = 0x8000,
    HTTPSRedirect = 0x010000,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLevel {
    Error = 0b0_0001_0010_0100_1000,
    Warning = 0b0_0001_0010_0100_1100,
    Info = 0b1_1101_1011_0101_1110,
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

    pub fn with_info(mut self, info: impl Into<Cow<'static, str>>) -> Self {
        self.info = Some(info.into());
        self
    }
}

impl From<EventType> for &'static str {
    fn from(kind: EventType) -> Self {
        match kind {
            EventType::ConnectionAttempt => "Connection attempted",
            EventType::ConnectionSuccess => "Connection successful",
            EventType::ConnectionDenied => "Connection denied",
            EventType::ConnectionFailed => "Connection failed",
            EventType::ConnectionClosed => "Connection closed",
            EventType::ThreadPoolProcessStarted => "Handler sent in thread pool",
            EventType::StreamDisconnectedWhileWaiting => {
                "Stream disconnected while waiting to be processed"
            }
            EventType::RequestAttempt => "Request attempted",
            EventType::RequestSuccess => "Request successful",
            EventType::RequestFailed => "Request failed",
            EventType::ResponseAttempt => "Response attempted",
            EventType::ResponseSuccess => "Response successful",
            EventType::ResponseFailed => "Response failed",
            EventType::KeepAliveRespected => "Connection kept alive after response",
            EventType::WebsocketConnectionRequested => "WebSocket connection requested",
            EventType::WebsocketConnectionClosed => "WebSocket connection closed",
            EventType::HTTPSRedirect => "Redirected to HTTPS",
        }
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
