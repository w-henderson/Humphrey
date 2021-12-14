pub mod app;
pub mod database;
pub mod error;
pub mod session;
pub mod user;

#[cfg(test)]
mod tests;

use crate::database::AuthDatabase;
use crate::error::AuthError;
use crate::session::Session;

pub use crate::user::User;

#[derive(Default)]
pub struct AuthProvider<T = Vec<User>>
where
    T: AuthDatabase,
{
    users: T,
}

impl<T> AuthProvider<T>
where
    T: AuthDatabase,
{
    pub fn new(users: T) -> Self {
        AuthProvider { users }
    }

    pub fn create_user(&mut self, password: impl AsRef<str>) -> Result<String, AuthError> {
        let new_user = User::create(password)?;
        self.users.add_user(new_user.clone())?;

        Ok(new_user.uid)
    }

    pub fn exists(&mut self, uid: impl AsRef<str>) -> bool {
        self.users.get_user_by_uid(&uid).is_some()
    }

    pub fn verify(&self, uid: impl AsRef<str>, password: impl AsRef<str>) -> bool {
        self.users
            .get_user_by_uid(&uid)
            .map(|user| user.verify(&password))
            .unwrap_or(false)
    }

    pub fn remove_user(&mut self, uid: impl AsRef<str>) -> Result<(), AuthError> {
        self.users.remove_user(&uid)
    }

    pub fn create_session(&mut self, uid: impl AsRef<str>) -> Result<String, AuthError> {
        let mut user = self
            .users
            .get_user_by_uid(uid.as_ref())
            .ok_or(AuthError::UserNotFound)?;

        if !user.session.map(|t| t.valid()).unwrap_or(false) {
            let token = Session::create();
            user.session = Some(token.clone());
            self.users.update_user(user)?;

            Ok(token.token)
        } else {
            Err(AuthError::SessionAlreadyExists)
        }
    }

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

    pub fn refresh_session(&mut self, token: impl AsRef<str>) -> Result<(), AuthError> {
        let mut user = self
            .users
            .get_user_by_token(token)
            .ok_or(AuthError::InvalidToken)?;

        let mut session = user.session.unwrap();
        session.refresh(3600);

        user.session = Some(session);
        self.users.update_user(user)?;

        Ok(())
    }

    pub fn invalidate_session(&mut self, token: impl AsRef<str>) {
        if let Some(mut user) = self.users.get_user_by_token(token) {
            user.session = None;
            self.users.update_user(user).unwrap();
        }
    }

    pub fn invalidate_user_session(&mut self, uid: impl AsRef<str>) {
        if let Some(mut user) = self.users.get_user_by_uid(uid) {
            user.session = None;
            self.users.update_user(user).unwrap();
        }
    }

    pub fn get_uid_by_token(&self, token: impl AsRef<str>) -> Result<String, AuthError> {
        self.users
            .get_user_by_token(token)
            .filter(|u| u.session.as_ref().unwrap().valid())
            .map(|user| user.uid)
            .ok_or(AuthError::InvalidToken)
    }
}
