//! <p align="center">
//!   <img src="https://raw.githubusercontent.com/w-henderson/Humphrey/master/assets/logo.png" width=250><br><br>
//!   <img src="https://img.shields.io/badge/language-rust-b07858?style=for-the-badge&logo=rust" style="margin-right:5px">
//!   <img src="https://img.shields.io/github/workflow/status/w-henderson/Humphrey/CI?style=for-the-badge" style="margin-right:5px">
//!   <img src="https://img.shields.io/crates/v/humphrey-auth?style=for-the-badge" style="margin-right:5px">
//! </p>
//!
//! # Authentication support for Humphrey.
//! Web applications commonly need a way of authenticating users. This crate provides an easy and secure way to do this, integrating with Humphrey using the `AuthApp` trait and allowing complete control over the database users are stored in. Humphrey Auth does not come with a database, but the `AuthDatabase` trait is implemented for `Vec<User>` to get started. For a production use, you should use a proper database and implement the `AuthDatabase` trait for it.
//!
//! **Note:** unlike the other crates in the Humphrey ecosystem, Humphrey Auth does require some dependencies, since fully-secure implementations of several complex cryptographic algorithms are required.
//!
//! ## Features
//! - Configurable and secure authentication using the Argon2 algorithm, provided by the [`argon2`](https://crates.io/crates/argon2) crate.
//! - Flexible user storage, allowing the use of any database by simply implementing a trait.
//! - Session and token management with a simple API.
//!
//! ## Installation
//! The Humphrey Auth crate can be installed by adding `humphrey_auth` to your dependencies in your `Cargo.toml` file.
//!
//! ## Documentation
//! The Humphrey Auth documentation can be found at [docs.rs](https://docs.rs/humphrey-auth/).
//!
//! ## Example
//! A basic example of username/password authentication can be found [here](https://github.com/w-henderson/Humphrey/tree/master/examples/auth).

#![warn(missing_docs)]

pub mod app;
pub mod config;
pub mod database;
pub mod error;
pub mod session;
pub mod user;

#[cfg(test)]
mod tests;

use crate::config::AuthConfig;
use crate::database::AuthDatabase;
use crate::error::AuthError;
use crate::session::Session;
use crate::user::User;

/// Represents an authentication provider.
/// Contains a database of users and provides methods for managing authentication.
///
/// If the database needs to be used from elsewhere in the program, it is advisable to
///   put the database behind an `Arc` and `Mutex`/`RwLock` and store a cloned reference
///   to the database in the users field of this struct.
#[derive(Default)]
pub struct AuthProvider<T>
where
    T: AuthDatabase,
{
    users: T,
    config: AuthConfig,
}

impl<T> AuthProvider<T>
where
    T: AuthDatabase,
{
    /// Create a new authentication provider with the given database.
    pub fn new(users: T) -> Self {
        AuthProvider {
            users,
            config: AuthConfig::default(),
        }
    }

    /// Use the given configuration for this authentication provider.
    pub fn with_config(mut self, config: AuthConfig) -> Self {
        self.config = config;
        self
    }

    /// Create a user with the given password. Returns the UID of the new user.
    pub fn create_user(&mut self, password: impl AsRef<str>) -> Result<String, AuthError> {
        let new_user = User::create(password, self.config.pepper.as_ref().map(|p| p.as_ref()))?;
        self.users.add_user(new_user.clone())?;

        Ok(new_user.uid)
    }

    /// Returns true if the user with the given UID exists.
    pub fn exists(&mut self, uid: impl AsRef<str>) -> bool {
        self.users.get_user_by_uid(&uid).is_some()
    }

    /// Verifies that the given password matches the password of the user with the given UID.
    pub fn verify(&self, uid: impl AsRef<str>, password: impl AsRef<str>) -> bool {
        self.users
            .get_user_by_uid(&uid)
            .map(|user| user.verify(&password, self.config.pepper.as_ref().map(|p| p.as_ref())))
            .unwrap_or(false)
    }

    /// Removes the user with the given UID.
    pub fn remove_user(&mut self, uid: impl AsRef<str>) -> Result<(), AuthError> {
        self.users.remove_user(&uid)
    }

    /// Creates a new session for the user with the given UID, returning the token.
    ///
    /// The session will expire after the configured duration.
    pub fn create_session(&mut self, uid: impl AsRef<str>) -> Result<String, AuthError> {
        let mut user = self
            .users
            .get_user_by_uid(uid.as_ref())
            .ok_or(AuthError::UserNotFound)?;

        if !user.session.map(|t| t.valid()).unwrap_or(false) {
            let token = Session::create_with_lifetime(self.config.default_lifetime);
            user.session = Some(token.clone());
            self.users.update_user(user)?;

            Ok(token.token)
        } else {
            Err(AuthError::SessionAlreadyExists)
        }
    }

    /// Creates a new session for the user with the given UID, returning the token.
    ///
    /// The session will expire after the given lifetime (in seconds).
    pub fn create_session_with_lifetime(
        &mut self,
        uid: impl AsRef<str>,
        lifetime: u64,
    ) -> Result<String, AuthError> {
        let mut user = self
            .users
            .get_user_by_uid(uid.as_ref())
            .ok_or(AuthError::UserNotFound)?;

        if !user.session.map(|t| t.valid()).unwrap_or(false) {
            let session = Session::create_with_lifetime(lifetime);
            user.session = Some(session.clone());
            self.users.update_user(user)?;

            Ok(session.token)
        } else {
            Err(AuthError::SessionAlreadyExists)
        }
    }

    /// Refreshes the session with the given token.
    /// If successful, the token will be set to expire after the configured duration.
    pub fn refresh_session(&mut self, token: impl AsRef<str>) -> Result<(), AuthError> {
        let mut user = self
            .users
            .get_user_by_token(token)
            .ok_or(AuthError::InvalidToken)?;

        let mut session = user.session.unwrap();
        session.refresh(self.config.default_refresh_lifetime);

        user.session = Some(session);
        self.users.update_user(user)?;

        Ok(())
    }

    /// Invalidates the given token, if it exists.
    pub fn invalidate_session(&mut self, token: impl AsRef<str>) {
        if let Some(mut user) = self.users.get_user_by_token(token) {
            user.session = None;
            self.users.update_user(user).unwrap();
        }
    }

    /// Invalidates the session of the user with the given UID, if they have one.
    pub fn invalidate_user_session(&mut self, uid: impl AsRef<str>) {
        if let Some(mut user) = self.users.get_user_by_uid(uid) {
            user.session = None;
            self.users.update_user(user).unwrap();
        }
    }

    /// Gets the UID of the user with the given token.
    pub fn get_uid_by_token(&self, token: impl AsRef<str>) -> Result<String, AuthError> {
        self.users
            .get_user_by_token(token)
            .filter(|u| u.session.as_ref().unwrap().valid())
            .map(|user| user.uid)
            .ok_or(AuthError::InvalidToken)
    }
}
