# Synchronous WebSocket
{{#title Synchronous WebSocket - Humphrey}}

Synchronous WebSocket applications call a user-specified handler for each client that connects, and the handler manages the connection until it closes. This means that the handler treats the connection like a regular stream, reading and writing data from it. While this is simpler and quicker, it also limits the number of simultaneous connections to the thread pool size of the underlying Humphrey application, since each connection is handled by a single thread.

This subsection will cover how to create a basic synchronous WebSocket server, as well as how to broadcast messages to all connected clients using an external crate. You'll see in the next subsection how to do these things asynchronously.