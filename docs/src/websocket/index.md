# Humphrey WebSocket
Humphrey WebSocket is a crate which extends Humphrey Core with WebSocket support by hooking into the latter's `WebsocketHandler` trait. It handles the WebSocket handshake and framing protocol and provides a simple and flexible API for sending and receiving messages. Using Humphrey's generic `Stream` type, it supports drop-in TLS. It also has no dependencies in accordance with Humphrey's goals of being dependency-free.

Humphrey WebSocket provides two ways to architect a WebSocket application: synchronous and asynchronous.

Synchronous WebSocket applications call user-specified handler functions when a client connects, and the handler function is expected to manage the connection until it closes. This is good for applications where you expect the connection to be short-lived and/or exchange a lot of data.

Asynchronous WebSocket applications call user-specified handler functions when a client connects, sends a message, or disconnects, and the handler function is only expected to manage the specific event that triggered it. This is more convenient for long-lived connections, as well as when broadcasting data to all connected clients is required, and vastly increases the number of concurrent WebSocket connections that can be handled.

This section of the guide will cover the following topics. We'll create the same example application in each of the two ways, so you can compare them by reading this section chronologically.

- [Synchronous WebSocket](sync/index.md)
  1. [Creating and running a basic WebSocket server](sync/getting-started.md)
  2. [Broadcasting messages to all connected clients](sync/broadcasting-messages.md)
- [Asynchronous WebSocket](async/index.md)
  1. [Creating and running a basic WebSocket server](async/getting-started.md)
  2. [Broadcasting messages to all connected clients](async/broadcasting-messages.md)
  3. [Using with an existing Humphrey App](async/linking.md)

It's recommended that you have basic familiarity with Rust and the [Humphrey Core](../core/index.md) crate before reading this section, as only Humphrey WebSocket-specific concepts are covered.
