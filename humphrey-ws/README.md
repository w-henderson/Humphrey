<div align="center">
  <img src="https://raw.githubusercontent.com/w-henderson/Humphrey/master/assets/logo.png" width=150>

  <h3 align="center">Humphrey WebSocket</h3>

  <p align="center">
    WebSocket support for the Humphrey web server.<br>
    <a href="https://humphrey.whenderson.dev/websocket/index.html">Guide</a> ·
    <a href="https://docs.rs/humphrey-ws">API Reference</a><br><br>
  </p>
</div>

<hr><br>

Humphrey WebSocket is a crate which extends Humphrey Core with WebSocket support by hooking into the latter's `WebsocketHandler` trait. It handles the WebSocket handshake and framing protocol and provides a simple and flexible API for sending and receiving messages. Using Humphrey's generic `Stream` type, it supports drop-in TLS. It also has no dependencies in accordance with Humphrey's goals of being dependency-free.

Learn more about Humphrey WebSocket [here](https://humphrey.whenderson.dev/websocket/index.html).