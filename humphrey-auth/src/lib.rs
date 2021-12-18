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

pub use crate::user::User;

/// Represents an authentication provider.
/// Contains a database of users and provides methods for managing authentication.
///
/// If the database needs to be used from elsewhere in the program, it is advisable to
///   put the database behind an `Arc` and `Mutex`/`RwLock` and store a cloned reference
///   to the database in the users field of this struct.
#[derive(Default)]
pub struct AuthProvider<T = Vec<User>>
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
