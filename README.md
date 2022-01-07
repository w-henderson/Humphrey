<div align="center">
  <img src="https://raw.githubusercontent.com/w-henderson/Humphrey/master/assets/logo.png" width=150>

  <h3 align="center">Humphrey</h3>

  <p align="center">
    A Performance-Focused, Dependency-Free Web Server.<br>
    <a href="#"><strong>Getting Started »</strong></a><br><br>
    <a href="https://github.com/w-henderson/Humphrey/blob/master/humphrey/README.md">Core Quickstart</a> ·
    <a href="https://github.com/w-henderson/Humphrey/blob/master/humphrey-server/README.md">Server Quickstart</a> ·
    <a href="https://github.com/w-henderson/Humphrey/blob/master/humphrey-ws/README.md">WebSocket Quickstart</a> ·
    <a href="https://github.com/w-henderson/Humphrey/blob/master/humphrey-auth/README.md">Auth Quickstart</a><br>
    <a href="https://docs.rs/humphrey">Core API Reference</a> ·
    <a href="https://docs.rs/humphrey-server">Server API Reference</a> ·
    <a href="https://docs.rs/humphrey-ws">WebSocket API Reference</a> ·
    <a href="https://docs.rs/humphrey-auth">Auth API Reference</a>
  </p><br>

  <img src="https://img.shields.io/badge/language-rust-b07858?style=for-the-badge&logo=rust" style="margin-right:5px">
  <img src="https://img.shields.io/github/workflow/status/w-henderson/Humphrey/CI?style=for-the-badge" style="margin-right:5px">
  <img src="https://img.shields.io/badge/dependencies-0-brightgreen?style=for-the-badge" style="margin-right:5px"><br>
  <img src="https://img.shields.io/crates/v/humphrey?label=core&style=for-the-badge" style="margin-right:5px">
  <img src="https://img.shields.io/crates/v/humphrey_server?label=server&style=for-the-badge" style="margin-right:5px">
  <img src="https://img.shields.io/crates/v/humphrey_ws?label=ws&style=for-the-badge" style="margin-right:5px">
  <img src="https://img.shields.io/crates/v/humphrey_auth?label=auth&style=for-the-badge" style="margin-right:5px"><br><br>
</div>

<hr><br>

Humphrey is a very fast, robust and flexible HTTP/1.1 web server, with support for static and dynamic content through its Rust crate and plugin system. It has no dependencies when only using default features, and the binary is easily extensible with a flexible configuration file and dynamically-loaded plugins. It also provides a WebSocket API for the easy integration of WebSockets into your application, and a simple authentication system for authenticating users and managing sessions.

## Contents
- [Use Humphrey as a crate](humphrey/README.md)
- [Use Humphrey as a standalone application](humphrey-server/README.md)
- [Use WebSockets with Humphrey](humphrey-ws/README.md)
- [Use authentication with Humphrey](humphrey-auth/README.md)
- [Use HTTPS/TLS with Humphrey](humphrey/TLS.md)