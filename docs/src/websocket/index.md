# Humphrey WebSocket
Humphrey WebSocket is a crate which extends Humphrey Core with WebSocket support by hooking into the latter's `WebsocketHandler` trait. It handles the WebSocket handshake and framing protocol and provides a simple and flexible API for sending and receiving messages. Using Humphrey's generic `Stream` type, it supports drop-in TLS. It also has no dependencies in accordance with Humphrey's goals of being dependency-free.

This section of the guide will cover the following topics:

1. [Creating and running a basic WebSocket server](getting-started.md)
2. [Broadcasting messages to all connected clients](broadcasting-messages.md)

It's recommended that you have basic familiarity with Rust and the [Humphrey Core](../core/index.md) crate before reading this section, as only Humphrey WebSocket-specific concepts are covered.