#![allow(clippy::new_without_default)]

use crate::message::Message;
use crate::restion::Restion;
use crate::stream::WebsocketStream;

use humphrey::stream::Stream;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// Represents an asynchronous WebSocket app.
pub struct AsyncWebsocketApp<State>
where
    State: Send + Sync + 'static,
{
    state: Arc<State>,
    streams: HashMap<SocketAddr, WebsocketStream<Stream>>,
    incoming_streams: Receiver<WebsocketStream<Stream>>,
    connect_hook: Arc<Mutex<Sender<WebsocketStream<Stream>>>>,
    on_connect: Option<Box<dyn EventHandler<State>>>,
    on_disconnect: Option<Box<dyn EventHandler<State>>>,
    on_message: Option<Box<dyn MessageHandler<State>>>,
}

/// Represents an asynchronous WebSocket stream.
pub struct AsyncStream {
    addr: SocketAddr,
    message_queue: Vec<Message>,
    connected: bool,
}

pub trait EventHandler<S>: Fn(&mut AsyncStream, Arc<S>) + Send + Sync + 'static {}
impl<T, S> EventHandler<S> for T where T: Fn(&mut AsyncStream, Arc<S>) + Send + Sync + 'static {}

pub trait MessageHandler<S>: Fn(&mut AsyncStream, Message, Arc<S>) + Send + Sync + 'static {}
impl<T, S> MessageHandler<S> for T where
    T: Fn(&mut AsyncStream, Message, Arc<S>) + Send + Sync + 'static
{
}

impl<State> AsyncWebsocketApp<State>
where
    State: Send + Sync + 'static,
{
    pub fn new() -> Self
    where
        State: Default,
    {
        let (connect_hook, incoming_streams) = channel();
        let connect_hook = Arc::new(Mutex::new(connect_hook));

        Self {
            state: Default::default(),
            streams: Default::default(),
            incoming_streams,
            connect_hook,
            on_connect: None,
            on_disconnect: None,
            on_message: None,
        }
    }

    pub fn connect_hook(&self) -> Arc<Mutex<Sender<WebsocketStream<Stream>>>> {
        self.connect_hook.clone()
    }

    pub fn on_connect(&mut self, handler: impl EventHandler<State>) {
        self.on_connect = Some(Box::new(handler));
    }

    pub fn on_disconnect(&mut self, handler: impl EventHandler<State>) {
        self.on_disconnect = Some(Box::new(handler));
    }

    pub fn on_message(&mut self, handler: impl MessageHandler<State>) {
        self.on_message = Some(Box::new(handler));
    }

    pub fn run(mut self) {
        loop {
            let keys: Vec<SocketAddr> = self.streams.keys().copied().collect();

            // Check for messages on each stream.
            for addr in keys {
                'inner: loop {
                    let stream = self.streams.get_mut(&addr).unwrap();

                    match stream.recv_nonblocking() {
                        Restion::Ok(message) => {
                            let mut async_stream = AsyncStream::new(addr);
                            if let Some(handler) = &self.on_message {
                                handler(&mut async_stream, message, self.state.clone());
                            }

                            for message in async_stream.into_inner() {
                                stream.send(message).unwrap();
                            }
                        }
                        Restion::Err(_) => {
                            let mut async_stream = AsyncStream::disconnected(addr);
                            if let Some(handler) = &self.on_disconnect {
                                handler(&mut async_stream, self.state.clone())
                            }

                            self.streams.remove(&addr);
                            break 'inner;
                        }
                        Restion::None => break 'inner,
                    }
                }
            }

            // Add any streams awaiting connection.
            for (addr, mut stream) in self
                .incoming_streams
                .try_iter()
                .filter_map(|s| s.peer_addr().map(|a| (a, s)).ok())
            {
                let mut async_stream = AsyncStream::new(addr);
                if let Some(handler) = &self.on_connect {
                    handler(&mut async_stream, self.state.clone());
                }

                for message in async_stream.into_inner() {
                    stream.send(message).unwrap();
                }

                self.streams.insert(addr, stream);
            }
        }
    }
}

impl AsyncStream {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            message_queue: vec![],
            connected: true,
        }
    }

    pub fn disconnected(addr: SocketAddr) -> Self {
        Self {
            addr,
            message_queue: vec![],
            connected: false,
        }
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn send(&mut self, message: Message) {
        assert!(self.connected);
        self.message_queue.push(message);
    }

    fn into_inner(self) -> Vec<Message> {
        self.message_queue
    }
}
