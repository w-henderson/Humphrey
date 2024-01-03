//! Provides asynchronous WebSocket functionality.

#![allow(clippy::new_without_default)]

use crate::handler::async_websocket_handler;
use crate::message::Message;
use crate::ping::Heartbeat;
use crate::restion::Restion;
use crate::stream::WebsocketStream;

use humphrey::thread::pool::ThreadPool;
use humphrey::App;

use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

/// Represents an asynchronous WebSocket app.
pub struct AsyncWebsocketApp<State, StreamState = ()>
where
    State: Send + Sync + 'static,
    StreamState: Send + Sync + Default + 'static,
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
    /// Ping configuration.
    heartbeat: Option<Heartbeat>,
    /// A hashmap with the addresses as the keys and the actual streams as the values.
    streams: HashMap<SocketAddr, StatefulWebsocketStream<StreamState>>,
    /// A receiver which is sent new streams to add to the hashmap.
    incoming_streams: Receiver<WebsocketStream>,
    /// A receiver which receives messages from handler threads to forward to clients.
    outgoing_messages: Receiver<OutgoingMessage>,
    /// A sender which is used by handler threads to send messages to clients.
    message_sender: Sender<OutgoingMessage>,
    /// The event handler called when a new client connects.
    on_connect: Option<Box<dyn EventHandler<State, StreamState>>>,
    /// The event handler called when a client disconnects.
    on_disconnect: Option<Box<dyn EventHandler<State, StreamState>>>,
    /// The event handler called when a client sends a message.
    on_message: Option<Box<dyn MessageHandler<State, StreamState>>>,
    /// Shutdown signal for the application.
    shutdown: Option<Receiver<()>>,
}

/// Represents a stateful WebSocket stream.
///
/// Encapsulates both the `WebsocketStream` and its state, which must be `Send + Sync + Default + 'static`.
pub struct StatefulWebsocketStream<StreamState = ()>
where
    StreamState: Send + Sync + Default + 'static,
{
    inner: WebsocketStream,
    state: Arc<StreamState>,
}

/// Represents an asynchronous WebSocket stream.
///
/// This is what is passed to the handler in place of the actual stream. It is able to send
///   messages back to the stream using the sender and the stream is identified by its address.
pub struct AsyncStream<StreamState = ()>
where
    StreamState: Send + Sync + Default + 'static,
{
    addr: SocketAddr,
    sender: Sender<OutgoingMessage>,
    /// The state of the stream.
    pub state: Arc<StreamState>,
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
    External(Arc<Mutex<Sender<WebsocketStream>>>),
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
pub trait EventHandler<S, S2: Send + Sync + Default + 'static>:
    Fn(AsyncStream<S2>, Arc<S>) + Send + Sync + 'static
{
}
impl<T, S, S2: Send + Sync + Default + 'static> EventHandler<S, S2> for T where
    T: Fn(AsyncStream<S2>, Arc<S>) + Send + Sync + 'static
{
}

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
pub trait MessageHandler<S, S2: Send + Sync + Default + 'static>:
    Fn(AsyncStream<S2>, Message, Arc<S>) + Send + Sync + 'static
{
}
impl<T, S, S2: Send + Sync + Default + 'static> MessageHandler<S, S2> for T where
    T: Fn(AsyncStream<S2>, Message, Arc<S>) + Send + Sync + 'static
{
}

impl<State, StreamState> AsyncWebsocketApp<State, StreamState>
where
    State: Send + Sync + 'static,
    StreamState: Send + Sync + Default + 'static,
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
            heartbeat: None,
            thread_pool: ThreadPool::new(32),
            streams: Default::default(),
            incoming_streams,
            outgoing_messages,
            message_sender,
            on_connect: None,
            on_disconnect: None,
            on_message: None,
            shutdown: None,
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
            heartbeat: None,
            thread_pool: ThreadPool::new(handler_threads),
            streams: Default::default(),
            incoming_streams,
            outgoing_messages,
            message_sender,
            on_connect: None,
            on_disconnect: None,
            on_message: None,
            shutdown: None,
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
            heartbeat: None,
            thread_pool: ThreadPool::new(32),
            streams: Default::default(),
            incoming_streams,
            outgoing_messages,
            message_sender,
            on_connect: None,
            on_disconnect: None,
            on_message: None,
            shutdown: None,
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
            heartbeat: None,
            thread_pool: ThreadPool::new(handler_threads),
            streams: Default::default(),
            incoming_streams,
            outgoing_messages,
            message_sender,
            on_connect: None,
            on_disconnect: None,
            on_message: None,
            shutdown: None,
        }
    }

    /// Returns a reference to the connection hook of the application.
    /// This is used by Humphrey Core to send new streams to the app.
    ///
    /// If the app is uses an internal Humphrey application, this will return `None`.
    pub fn connect_hook(&self) -> Option<Arc<Mutex<Sender<WebsocketStream>>>> {
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
    pub fn on_connect(&mut self, handler: impl EventHandler<State, StreamState>) {
        self.on_connect = Some(Box::new(handler));
    }

    /// Set the event handler called when a client disconnects.
    pub fn on_disconnect(&mut self, handler: impl EventHandler<State, StreamState>) {
        self.on_disconnect = Some(Box::new(handler));
    }

    /// Set the message handler called when a client sends a message.
    pub fn on_message(&mut self, handler: impl MessageHandler<State, StreamState>) {
        self.on_message = Some(Box::new(handler));
    }

    /// Set the event handler called when a new client connects.
    /// Returns itself for use in a builder pattern.
    pub fn with_connect_handler(mut self, handler: impl EventHandler<State, StreamState>) -> Self {
        self.on_connect(handler);
        self
    }

    /// Set the event handler called when a client disconnects.
    /// Returns itself for use in a builder pattern.
    pub fn with_disconnect_handler(
        mut self,
        handler: impl EventHandler<State, StreamState>,
    ) -> Self {
        self.on_disconnect(handler);
        self
    }

    /// Set the message handler called when a client sends a message.
    /// Returns itself for use in a builder pattern.
    pub fn with_message_handler(
        mut self,
        handler: impl MessageHandler<State, StreamState>,
    ) -> Self {
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

    /// Sets the heartbeat configuration for the async app.
    ///
    /// By default, this is off, meaning the app will not send heartbeats. If your application needs to detect
    ///   disconnections which occur suddenly, as in without sending a "close" frame, you should set this up.
    ///   It is particularly useful for detecting disconnections caused by network issues, which would not be ordinarily
    ///   detected by the client.
    pub fn with_heartbeat(mut self, heartbeat: Heartbeat) -> Self {
        self.heartbeat = Some(heartbeat);
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

        let mut last_ping = Instant::now();

        loop {
            if let Some(ref s) = self.shutdown {
                if s.try_recv().is_ok() {
                    break;
                }
            }

            let keys: Vec<SocketAddr> = self.streams.keys().copied().collect();

            // Calculate whether a ping should be sent this iteration.
            let will_ping = self
                .heartbeat
                .as_ref()
                .map(|config| {
                    let will_ping = last_ping.elapsed() >= config.interval;

                    if will_ping {
                        last_ping = Instant::now();
                    }

                    will_ping
                })
                .unwrap_or(false);

            // Check for messages and status on each stream.
            for addr in keys {
                'inner: loop {
                    let stream = self.streams.get_mut(&addr).unwrap();

                    match stream.inner.recv_nonblocking() {
                        Restion::Ok(message) => {
                            if let Some(handler) = &message_handler {
                                let async_stream = AsyncStream::new(
                                    addr,
                                    self.message_sender.clone(),
                                    stream.state.clone(),
                                );

                                let cloned_state = self.state.clone();
                                let cloned_handler = handler.clone();

                                self.thread_pool.execute(move || {
                                    (cloned_handler)(async_stream, message, cloned_state)
                                });
                            }
                        }
                        Restion::Err(_) => {
                            if let Some(handler) = &disconnect_handler {
                                let async_stream = AsyncStream::disconnected(
                                    addr,
                                    self.message_sender.clone(),
                                    stream.state.clone(),
                                );

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

                if let Some(stream) = self.streams.get_mut(&addr) {
                    // If the stream has timed out without sending a close frame, process it as a disconnection.
                    if let Some(ping) = &self.heartbeat {
                        if stream.inner.last_pong.elapsed() >= ping.timeout {
                            if let Some(handler) = &disconnect_handler {
                                let async_stream = AsyncStream::disconnected(
                                    addr,
                                    self.message_sender.clone(),
                                    stream.state.clone(),
                                );

                                let cloned_state = self.state.clone();
                                let cloned_handler = handler.clone();

                                self.thread_pool
                                    .execute(move || (cloned_handler)(async_stream, cloned_state));
                            }

                            self.streams.remove(&addr);
                            continue;
                        }
                    }

                    // If a ping is due, send one.
                    if will_ping {
                        stream.inner.ping().ok();
                    }
                }
            }

            // Add any streams awaiting connection.
            for (addr, stream) in self
                .incoming_streams
                .try_iter()
                .filter_map(|s| s.peer_addr().map(|a| (a, s)).ok())
            {
                let stream_state = Arc::new(StreamState::default());

                if let Some(handler) = &connect_handler {
                    let async_stream =
                        AsyncStream::new(addr, self.message_sender.clone(), stream_state.clone());
                    let cloned_state = self.state.clone();
                    let cloned_handler = handler.clone();

                    self.thread_pool.execute(move || {
                        (cloned_handler)(async_stream, cloned_state);
                    });
                }

                self.streams.insert(
                    addr,
                    StatefulWebsocketStream {
                        inner: stream,
                        state: stream_state,
                    },
                );
            }

            for message in self.outgoing_messages.try_iter() {
                match message {
                    OutgoingMessage::Message(addr, message) => {
                        if let Some(stream) = self.streams.get_mut(&addr) {
                            // Ignore errors with sending for now, and deal with them in the next iteration.
                            stream.inner.send(message).ok();
                        }
                    }
                    OutgoingMessage::Broadcast(message) => {
                        let frame = message.to_frame();
                        for stream in self.streams.values_mut() {
                            // Ignore errors with sending for now, and deal with them in the next iteration.
                            stream.inner.send_raw(&frame).ok();
                        }
                    }
                }
            }

            if let Some(interval) = self.poll_interval {
                sleep(interval);
            }
        }
        self.thread_pool.stop();
    }

    /// Registers a shutdown signal to gracefully shutdown the app
    pub fn with_shutdown(mut self, shutdown_receiver: Receiver<()>) -> Self {
        self.shutdown = Some(shutdown_receiver);
        self
    }
}

impl<StreamState> AsyncStream<StreamState>
where
    StreamState: Send + Sync + Default + 'static,
{
    /// Create a new asynchronous stream.
    pub fn new(addr: SocketAddr, sender: Sender<OutgoingMessage>, state: Arc<StreamState>) -> Self {
        Self {
            addr,
            sender,
            state,
            connected: true,
        }
    }

    /// Create a new disconnected asynchronous stream.
    /// This is used for getting the address of a disconnected stream.
    pub fn disconnected(
        addr: SocketAddr,
        sender: Sender<OutgoingMessage>,
        state: Arc<StreamState>,
    ) -> Self {
        Self {
            addr,
            sender,
            state,
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
