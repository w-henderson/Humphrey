//! Provides asynchronous WebSocket functionality.

#![allow(clippy::new_without_default)]

use crate::handler::async_websocket_handler;
use crate::message::Message;
use crate::restion::Restion;
use crate::stream::WebsocketStream;

use humphrey::stream::Stream;
use humphrey::thread::pool::ThreadPool;
use humphrey::App;

use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration;

/// Represents an asynchronous WebSocket app.
pub struct AsyncWebsocketApp<State>
where
    State: Send + Sync + 'static,
{
    /// Represents the link to a Humphrey application.
    ///
    /// This may be:
    /// - `HumphreyLink::Internal`, in which case the app uses its own internal Humphrey application
    /// - `HumphreyLink::External`, in which case the app is linked to an external Humphrey application and receives connections through a channel
    ///
    /// Each enum variant has corresponding fields for the configuration.
    humphrey_link: HumphreyLink,
    /// Represents the state of the application.
    state: Arc<State>,
    /// The internal thread pool of the application.
    thread_pool: ThreadPool,
    /// The amount of time between polling.
    poll_interval: Option<Duration>,
    /// A hashmap with the addresses as the keys and the actual streams as the values.
    streams: HashMap<SocketAddr, WebsocketStream<Stream>>,
    /// A receiver which is sent new streams to add to the hashmap.
    incoming_streams: Receiver<WebsocketStream<Stream>>,
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

/// Represents a global sender which can send messages to clients without waiting for events.
pub struct AsyncSender(Sender<OutgoingMessage>);

/// Represents a message to be sent from the server to a client.
pub enum OutgoingMessage {
    /// A message to be sent to a specific client.
    Message(SocketAddr, Message),
    /// A message to be sent to every connected client.
    Broadcast(Message),
}

/// Represents the link to a Humphrey application.
///
/// This may be:
/// - `HumphreyLink::Internal`, in which case the app uses its own internal Humphrey application
/// - `HumphreyLink::External`, in which case the app is linked to an external Humphrey application and receives connections through a channel
///
/// Each enum variant has corresponding fields for the configuration.
pub enum HumphreyLink {
    /// The app uses its own internal Humphrey application.
    Internal(Box<App>, SocketAddr),
    /// The app is linked to an external Humphrey application and receives connections through a channel.
    External(Arc<Mutex<Sender<WebsocketStream<Stream>>>>),
}

/// Represents a function able to handle a WebSocket event (a connection or disconnection).
/// It is passed the stream which triggered the event as well as the app's state.
///
/// ## Example
/// A basic example of an event handler would be as follows:
/// ```
/// fn connection_handler(stream: AsyncStream, state: Arc<()>) {
///     println!("A new client connected! {:?}", stream.addr);
///
///     stream.send(Message::new("Hello, World!"));
/// }
/// ```
pub trait EventHandler<S>: Fn(AsyncStream, Arc<S>) + Send + Sync + 'static {}
impl<T, S> EventHandler<S> for T where T: Fn(AsyncStream, Arc<S>) + Send + Sync + 'static {}

/// Represents a function able to handle a message event.
/// It is passed the stream which sent the message, the message and the app's state.
///
/// ## Example
/// A basic example of a message handler would be as follows:
/// ```
/// fn message_handler(stream: AsyncStream, message: Message, state: Arc<()>) {
///    println!("A message was received from {:?}: {}", stream.addr, message.text().unwrap());
///
///    stream.send(Message::new("Message received."));
/// }
/// ```
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

        let humphrey_app = App::new_with_config(1, ())
            .with_websocket_route("/*", async_websocket_handler(connect_hook));

        Self {
            humphrey_link: HumphreyLink::Internal(
                Box::new(humphrey_app),
                "0.0.0.0:80".to_socket_addrs().unwrap().next().unwrap(),
            ),
            state: Default::default(),
            poll_interval: Some(Duration::from_millis(10)),
            thread_pool: ThreadPool::new(32),
            streams: Default::default(),
            incoming_streams,
            outgoing_messages,
            message_sender,
            on_connect: None,
            on_disconnect: None,
            on_message: None,
        }
    }

    /// Creates a new asynchronous WebSocket app with a custom state and configuration.
    ///
    /// - `state`: The state of the application.
    /// - `handler_threads`: The size of the handler thread pool.
    /// - `connection_threads`: The size of the connection handler thread pool (the underlying Humphrey app).
    pub fn new_with_config(
        state: State,
        handler_threads: usize,
        connection_threads: usize,
    ) -> Self {
        let (connect_hook, incoming_streams) = channel();
        let connect_hook = Arc::new(Mutex::new(connect_hook));

        let (message_sender, outgoing_messages) = channel();

        let humphrey_app = App::new_with_config(connection_threads, ())
            .with_websocket_route("/*", async_websocket_handler(connect_hook));

        Self {
            humphrey_link: HumphreyLink::Internal(
                Box::new(humphrey_app),
                "0.0.0.0:80".to_socket_addrs().unwrap().next().unwrap(),
            ),
            state: Arc::new(state),
            poll_interval: Some(Duration::from_millis(10)),
            thread_pool: ThreadPool::new(handler_threads),
            streams: Default::default(),
            incoming_streams,
            outgoing_messages,
            message_sender,
            on_connect: None,
            on_disconnect: None,
            on_message: None,
        }
    }

    /// Creates a new asynchronous WebSocket app without creating a Humphrey application.
    ///
    /// This is useful if you want to use the app as part of a Humphrey application, or if you want to use TLS.
    ///
    /// You'll need to manually link the app to a Humphrey application using the `connect_hook`.
    pub fn new_unlinked() -> Self
    where
        State: Default,
    {
        let (connect_hook, incoming_streams) = channel();
        let connect_hook = Arc::new(Mutex::new(connect_hook));

        let (message_sender, outgoing_messages) = channel();

        Self {
            humphrey_link: HumphreyLink::External(connect_hook),
            state: Default::default(),
            poll_interval: Some(Duration::from_millis(10)),
            thread_pool: ThreadPool::new(32),
            streams: Default::default(),
            incoming_streams,
            outgoing_messages,
            message_sender,
            on_connect: None,
            on_disconnect: None,
            on_message: None,
        }
    }

    /// Creates a new asynchronous WebSocket app with a custom state and configuration, without creating a Humphrey application.
    ///
    /// This is useful if you want to use the app as part of a Humphrey application, or if you want to use TLS.
    ///
    /// You'll need to manually link the app to a Humphrey application using the `connect_hook`.
    ///
    /// - `state`: The state of the application.
    /// - `handler_threads`: The size of the handler thread pool.
    pub fn new_unlinked_with_config(state: State, handler_threads: usize) -> Self {
        let (connect_hook, incoming_streams) = channel();
        let connect_hook = Arc::new(Mutex::new(connect_hook));

        let (message_sender, outgoing_messages) = channel();

        Self {
            humphrey_link: HumphreyLink::External(connect_hook),
            state: Arc::new(state),
            poll_interval: Some(Duration::from_millis(10)),
            thread_pool: ThreadPool::new(handler_threads),
            streams: Default::default(),
            incoming_streams,
            outgoing_messages,
            message_sender,
            on_connect: None,
            on_disconnect: None,
            on_message: None,
        }
    }

    /// Returns a reference to the connection hook of the application.
    /// This is used by Humphrey Core to send new streams to the app.
    ///
    /// If the app is uses an internal Humphrey application, this will return `None`.
    pub fn connect_hook(&self) -> Option<Arc<Mutex<Sender<WebsocketStream<Stream>>>>> {
        match &self.humphrey_link {
            HumphreyLink::External(connect_hook) => Some(connect_hook.clone()),
            _ => None,
        }
    }

    /// Returns a new `AsyncSender`, which can be used to send messages.
    pub fn sender(&self) -> AsyncSender {
        AsyncSender(self.message_sender.clone())
    }

    /// Gets a reference to the app’s state. This should only be used in the main thread, as the state is passed to event handlers otherwise.
    pub fn get_state(&self) -> Arc<State> {
        self.state.clone()
    }

    /// Set the event handler called when a new client connects.
    pub fn on_connect(&mut self, handler: impl EventHandler<State>) {
        self.on_connect = Some(Box::new(handler));
    }

    /// Set the event handler called when a client disconnects.
    pub fn on_disconnect(&mut self, handler: impl EventHandler<State>) {
        self.on_disconnect = Some(Box::new(handler));
    }

    /// Set the message handler called when a client sends a message.
    pub fn on_message(&mut self, handler: impl MessageHandler<State>) {
        self.on_message = Some(Box::new(handler));
    }

    /// Set the event handler called when a new client connects.
    /// Returns itself for use in a builder pattern.
    pub fn with_connect_handler(mut self, handler: impl EventHandler<State>) -> Self {
        self.on_connect(handler);
        self
    }

    /// Set the event handler called when a client disconnects.
    /// Returns itself for use in a builder pattern.
    pub fn with_disconnect_handler(mut self, handler: impl EventHandler<State>) -> Self {
        self.on_disconnect(handler);
        self
    }

    /// Set the message handler called when a client sends a message.
    /// Returns itself for use in a builder pattern.
    pub fn with_message_handler(mut self, handler: impl MessageHandler<State>) -> Self {
        self.on_message(handler);
        self
    }

    /// Set the address to run the application on.
    /// Returns itself for use in a builder pattern.
    ///
    /// This function has no effect if the app does not manage its own internal Humphrey application.
    pub fn with_address<T>(mut self, address: T) -> Self
    where
        T: ToSocketAddrs,
    {
        self.humphrey_link = match self.humphrey_link {
            HumphreyLink::Internal(app, _) => {
                let address = address.to_socket_addrs().unwrap().next().unwrap();
                HumphreyLink::Internal(app, address)
            }
            HumphreyLink::External(connect_hook) => HumphreyLink::External(connect_hook),
        };
        self
    }

    /// Sets the polling interval of the async app.
    ///
    /// By default, this is 10ms, meaning the app will check for new events 100 times a second.
    pub fn with_polling_interval(mut self, interval: Option<Duration>) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Start the application on the main thread.
    pub fn run(mut self) {
        // Ensure that the underlying Humphrey application is running if it is internal.
        if let HumphreyLink::Internal(app, addr) = self.humphrey_link {
            spawn(move || app.run(addr).unwrap());
        }

        self.thread_pool.start();

        let connect_handler = self.on_connect.map(Arc::new);
        let disconnect_handler = self.on_disconnect.map(Arc::new);
        let message_handler = self.on_message.map(Arc::new);

        loop {
            let keys: Vec<SocketAddr> = self.streams.keys().copied().collect();

            // Check for messages on each stream.
            for addr in keys {
                'inner: loop {
                    let stream = self.streams.get_mut(&addr).unwrap();

                    match stream.recv_nonblocking() {
                        Restion::Ok(message) => {
                            if let Some(handler) = &message_handler {
                                let async_stream =
                                    AsyncStream::new(addr, self.message_sender.clone());
                                let cloned_state = self.state.clone();
                                let cloned_handler = handler.clone();

                                self.thread_pool.execute(move || {
                                    (cloned_handler)(async_stream, message, cloned_state)
                                });
                            }
                        }
                        Restion::Err(_) => {
                            if let Some(handler) = &disconnect_handler {
                                let async_stream =
                                    AsyncStream::disconnected(addr, self.message_sender.clone());
                                let cloned_state = self.state.clone();
                                let cloned_handler = handler.clone();

                                self.thread_pool
                                    .execute(move || (cloned_handler)(async_stream, cloned_state));
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
                if let Some(handler) = &connect_handler {
                    let async_stream = AsyncStream::new(addr, self.message_sender.clone());
                    let cloned_state = self.state.clone();
                    let cloned_handler = handler.clone();

                    self.thread_pool.execute(move || {
                        (cloned_handler)(async_stream, cloned_state);
                    });
                }

                self.streams.insert(addr, stream);
            }

            for message in self.outgoing_messages.try_iter() {
                match message {
                    OutgoingMessage::Message(addr, message) => {
                        if let Some(stream) = self.streams.get_mut(&addr) {
                            // Ignore errors with sending for now, and deal with them in the next iteration.
                            stream.send(message).ok();
                        }
                    }
                    OutgoingMessage::Broadcast(message) => {
                        let frame = message.to_frame();
                        for stream in self.streams.values_mut() {
                            // Ignore errors with sending for now, and deal with them in the next iteration.
                            stream.send_raw(&frame).ok();
                        }
                    }
                }
            }

            if let Some(interval) = self.poll_interval {
                sleep(interval);
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
            .ok();
    }

    /// Broadcast a message to all connected clients.
    pub fn broadcast(&self, message: Message) {
        self.sender.send(OutgoingMessage::Broadcast(message)).ok();
    }

    /// Get the address of the stream.
    pub fn peer_addr(&self) -> SocketAddr {
        self.addr
    }
}

impl AsyncSender {
    /// Send a message to the client identified by the socket address.
    pub fn send(&self, address: SocketAddr, message: Message) {
        self.0.send(OutgoingMessage::Message(address, message)).ok();
    }

    /// Broadcast a message to all connected clients.
    pub fn broadcast(&self, message: Message) {
        self.0.send(OutgoingMessage::Broadcast(message)).ok();
    }
}
