# Introduction

Humphrey is a very fast, robust and flexible HTTP/1.1 web server. It provides an executable web server, similar to Nginx, a Rust crate for building your own web applications, first-party WebSocket support, and a simple authentication system. In this guide, you'll get a strong understanding of how to use and build upon all of these components.

The executable web server component of the project is often referred to as "Humphrey Server", and you can learn how to install, configure and run it [here](server/index.md). It also supports plugins, which provide limitless extensibility of the server and the creation of which is also covered in this guide.

The underlying Rust crate is often referred to as "Humphrey Core", and provides a framework for building web applications, with the ability to act as both a client and a server. You can learn how to set up and build your own web application using Humphrey Core [here](core/index.md).

The WebSocket functionality is provided by a separate crate, often referred to as "Humphrey WebSocket", which integrates with the core crate for ease of development. You can learn how to Humphrey WebSocket in your own application [here](websocket/index.md).

Humphrey also provides a simple JSON library called "Humphrey JSON". It allows for the manipulation of JSON data in a variety of ways. You can learn how to use Humphrey JSON [here](json/index.md).

The simple authentication system is also provided by a separate crate, often referred to as "Humphrey Auth", which extends the core crate with authentication-related features. You can learn how to use Humphrey Auth in your own application [here](auth/index.md).

## Quick Reference
- [Setting up Humphrey Server](server/getting-started.md)
- [A basic web application using Humphrey Core](core/getting-started.md)
- [Using WebSocket with Humphrey Core](websocket/sync/getting-started.md)
- [Using Humphrey as a Client](core/client.md)
- [Using PHP with Humphrey Server](server/using-php.md)
- [Creating a Humphrey Server plugin](server/creating-a-plugin.md)
- [Using Humphrey JSON](json/index.md)

## Latest Versions
This book is up-to-date with the following crate versions.

| Crate | Version |
| ----- | ------- |
| Humphrey Core | 0.5.4 |
| Humphrey Server | 0.5.0 |
| Humphrey WebSocket | 0.3.0 |
| Humphrey JSON | 0.1.0 |
| Humphrey Auth | 0.1.3 |