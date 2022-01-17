# Humphrey Auth
Humphrey Auth is simple authentication crate for Humphrey applications. It provides a simple and database-agnostic way to authenticate users and handle sessions and tokens. It does depend upon the [`argon2`](https://docs.rs/argon2), [`uuid`](https://docs.rs/uuid) and [`rand_core`](https://docs.rs/rand_core) crates to ensure that it is secure.

This section of the guide will cover the following topics:

1. [Setting up user authentication](getting-started.md)
2. [Integrating with a database](database.md)

This section requires knowledge of the [Humphrey Core](../core/index.md) crate and the Rust programming language.