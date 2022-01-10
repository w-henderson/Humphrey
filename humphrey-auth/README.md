<p align="center">
  <img src="https://raw.githubusercontent.com/w-henderson/Humphrey/master/assets/logo.png" width=250><br><br>
  <img src="https://img.shields.io/badge/language-rust-b07858?style=for-the-badge&logo=rust" style="margin-right:5px">
  <img src="https://img.shields.io/github/workflow/status/w-henderson/Humphrey/CI?style=for-the-badge" style="margin-right:5px">
  <img src="https://img.shields.io/crates/v/humphrey-auth?style=for-the-badge" style="margin-right:5px">
</p>

# Authentication support for Humphrey.
Web applications commonly need a way of authenticating users. This crate provides an easy and secure way to do this, integrating with Humphrey using the `AuthApp` trait and allowing complete control over the database users are stored in. Humphrey Auth does not come with a database, but the `AuthDatabase` trait is implemented for `Vec<User>` to get started. For a production use, you should use a proper database and implement the `AuthDatabase` trait for it.

**Note:** unlike the other crates in the Humphrey ecosystem, Humphrey Auth does require some dependencies, since fully-secure implementations of several complex cryptographic algorithms are required.

## Features
- Configurable and secure authentication using the Argon2 algorithm, provided by the [`argon2`](https://crates.io/crates/argon2) crate.
- Flexible user storage, allowing the use of any database by simply implementing a trait.
- Session and token management with a simple API.

## Installation
The Humphrey Auth crate can be installed by adding `humphrey_auth` to your dependencies in your `Cargo.toml` file.

## Documentation
The Humphrey Auth documentation can be found at [docs.rs](https://docs.rs/humphrey-auth/).

## Example
A basic example of username/password authentication can be found [here](https://github.com/w-henderson/Humphrey/tree/master/examples/auth).