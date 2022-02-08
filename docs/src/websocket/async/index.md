# Asynchronous WebSocket
{{#title Asynchronous WebSocket - Humphrey}}

For applications which serve many clients at once, synchronous approaches can be a bottleneck. Humphrey WebSocket's second option for building a WebSocket application is asynchronously, which entails using event handlers for specific events (connection, disconnection and messages).

This subsection of the guide will cover how to create a basic asynchronous WebSocket server, as well as how to broadcast messages to all connected clients. We'll also compare this approach to the previous one as we build the same example application.