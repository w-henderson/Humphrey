# Introduction

Humphrey is a very fast, robust and flexible HTTP/1.1 web server. It provides an executable web server, similar to Nginx, a Rust crate for building your own web applications, and WebSocket functionality built in. In this guide, you'll get a strong understanding of how to use and build upon all of these components.

The executable web server component of the project is often referred to as "Humphrey Server", and you can learn how to install, configure and run it [here](server/index.md). It also supports plugins, which provide limitless extensibility of the server and the creation of which is also covered in this guide.

The underlying Rust crate is often referred to as "Humphrey Core", and provides a framework for building web applications. You can learn how to set up and build your own web application using Humphrey Core [here](core/index.md).

The WebSocket functionality is provided by a separate crate, often referred to as "Humphrey WebSocket", which integrates with the core crate for ease of development. You can learn how to Humphrey WebSocket in your own application [here](websocket/index.md).

## Quick Reference
- [Setting up Humphrey Server](server/getting-started.md)
- [A basic web application using Humphrey Core](core/getting-started.md)
- [Using WebSocket with Humphrey Core](websocket/getting-started.md)
- [Using PHP with Humphrey Server](server/using-php.md)
- [Creating a Humphrey Server plugin](server/plugin/getting-started.md)