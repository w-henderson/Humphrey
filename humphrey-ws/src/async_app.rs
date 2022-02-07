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
    /// Represents the state of the application.
    state: Arc<State>,
    /// A hashmap with the addresses as the keys and the actual streams as the values.
    streams: HashMap<SocketAddr, WebsocketStream<Stream>>,
    /// A receiver which is sent new streams to add to the hashmap.
    incoming_streams: Receiver<WebsocketStream<Stream>>,
    /// A sender which is used by Humphrey Core to send new streams to the async app.
    connect_hook: Arc<Mutex<Sender<WebsocketStream<Stream>>>>,
    /// A receiver which receives messages from handler threads to forward to clients.
    outgoing_messages: Receiver<OutgoingMessage>,
    /// A sender which is used by handler threads to send messages to clients.
    message_sender: Sender<OutgoingMessage>,
    /// The event handler called when a new client connects.
    on_connect: Option<Box<dyn EventHandler<State>>>,
    /// The event handler called when a client disconnects.
    on_disconnect: Option<Box<dyn EventHandler<State>>>,
    /// The event handler called when a client sends a message.
    on_message: Option<Box<dyn MessageHandler<State>>>,
}

/// Represents an asynchronous WebSocket stream.
///
/// This is what is passed to the handler in place of the actual stream. It is able to send
///   messages back to the stream using the sender and the stream is identified by its address.
pub struct AsyncStream {
    addr: SocketAddr,
    sender: Sender<OutgoingMessage>,
    connected: bool,
}

/// Represents a message to be sent from the server to a client.
pub enum OutgoingMessage {
    /// A message to be sent to a specific client.
    Message(SocketAddr, Message),
    /// A message to be sent to every connected client.
    Broadcast(Message),
}

pub trait EventHandler<S>: Fn(AsyncStream, Arc<S>) + Send + Sync + 'static {}
impl<T, S> EventHandler<S> for T where T: Fn(AsyncStream, Arc<S>) + Send + Sync + 'static {}

pub trait MessageHandler<S>: Fn(AsyncStream, Message, Arc<S>) + Send + Sync + 'static {}
impl<T, S> MessageHandler<S> for T where T: Fn(AsyncStream, Message, Arc<S>) + Send + Sync + 'static {}

impl<State> AsyncWebsocketApp<State>
where
    State: Send + Sync + 'static,
{
    /// Creates a new asynchronous WebSocket app.
    pub fn new() -> Self
    where
        State: Default,
    {
        let (connect_hook, incoming_streams) = channel();
        let connect_hook = Arc::new(Mutex::new(connect_hook));

        let (message_sender, outgoing_messages) = channel();

        Self {
            state: Default::default(),
            streams: Default::default(),
            incoming_streams,
            connect_hook,
            outgoing_messages,
            message_sender,
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
                            let async_stream = AsyncStream::new(addr, self.message_sender.clone());
                            if let Some(handler) = &self.on_message {
                                handler(async_stream, message, self.state.clone());
                            }
                        }
                        Restion::Err(_) => {
                            let async_stream =
                                AsyncStream::disconnected(addr, self.message_sender.clone());
                            if let Some(handler) = &self.on_disconnect {
                                handler(async_stream, self.state.clone());
                            }

                            self.streams.remove(&addr);
                            break 'inner;
                        }
                        Restion::None => break 'inner,
                    }
                }
            }

            // Add any streams awaiting connection.
            for (addr, stream) in self
                .incoming_streams
                .try_iter()
                .filter_map(|s| s.peer_addr().map(|a| (a, s)).ok())
            {
                let async_stream = AsyncStream::new(addr, self.message_sender.clone());
                if let Some(handler) = &self.on_connect {
                    handler(async_stream, self.state.clone());
                }

                self.streams.insert(addr, stream);
            }

            for message in self.outgoing_messages.try_iter() {
                match message {
                    OutgoingMessage::Message(addr, message) => {
                        if let Some(stream) = self.streams.get_mut(&addr) {
                            stream.send(message).unwrap();
                        }
                    }
                    OutgoingMessage::Broadcast(message) => {
                        let frame = message.to_frame();
                        for stream in self.streams.values_mut() {
                            stream.send_raw(&frame).unwrap();
                        }
                    }
                }
            }
        }
    }
}

impl AsyncStream {
    /// Create a new asynchronous stream.
    pub fn new(addr: SocketAddr, sender: Sender<OutgoingMessage>) -> Self {
        Self {
            addr,
            sender,
            connected: true,
        }
    }

    /// Create a new disconnected asynchronous stream.
    /// This is used for getting the address of a disconnected stream.
    pub fn disconnected(addr: SocketAddr, sender: Sender<OutgoingMessage>) -> Self {
        Self {
            addr,
            sender,
            connected: false,
        }
    }

    /// Send a message to the client.
    pub fn send(&self, message: Message) {
        assert!(self.connected);
        self.sender
            .send(OutgoingMessage::Message(self.addr, message))
            .unwrap();
    }

    /// Broadcast a message to all connected clients.
    pub fn broadcast(&self, message: Message) {
        assert!(self.connected);
        self.sender
            .send(OutgoingMessage::Broadcast(message))
            .unwrap();
    }

    /// Get the address of the stream.
    pub fn peer_addr(&self) -> SocketAddr {
        self.addr
    }
}
