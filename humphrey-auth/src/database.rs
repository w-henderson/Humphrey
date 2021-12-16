use crate::error::AuthError;
use crate::session::Session;
use crate::user::User;

/// Represents a database which can be used to store auth information.
/// Must be implemented for whatever database you are using.
///
/// It is good practice to have a separate collection for authentication information and
///   user details, so whatever collection/table you use in your implementation of this trait
///   should ideally not be used for anything else.
pub trait AuthDatabase {
    /// Returns the user associated with the given UID, or `None` if not found.
    fn get_user_by_uid(&self, uid: impl AsRef<str>) -> Option<User>;
    /// Returns the user who owns the given token, or `None` if not found.
    fn get_user_by_token(&self, token: impl AsRef<str>) -> Option<User>;
    /// Returns the session identifed by the given token, or `None` if not found.
    fn get_session_by_token(&self, token: impl AsRef<str>) -> Option<Session>;

    /// Update the user in the database.
    /// The user should be identified by their UID.
    fn update_user(&mut self, user: User) -> Result<(), AuthError>;
    /// Add a user to the database.
    fn add_user(&mut self, user: User) -> Result<(), AuthError>;
    /// Remove the user with the given UID from the database.
    fn remove_user(&mut self, uid: impl AsRef<str>) -> Result<(), AuthError>;
}

impl AuthDatabase for Vec<User> {
    fn get_user_by_uid(&self, uid: impl AsRef<str>) -> Option<User> {
        self.iter()
            .find(|user| user.uid == uid.as_ref())
            .map(|user| (*user).clone())
    }

    fn get_user_by_token(&self, token: impl AsRef<str>) -> Option<User> {
        self.iter()
            .find(|u| u.session.is_some() && u.session.as_ref().unwrap().token == token.as_ref())
            .map(|user| (*user).clone())
    }

    fn get_session_by_token(&self, token: impl AsRef<str>) -> Option<Session> {
        self.iter()
            .find(|u| u.session.is_some() && u.session.as_ref().unwrap().token == token.as_ref())
            .map(|user| user.session.as_ref().unwrap().clone())
    }

    fn update_user(&mut self, user: User) -> Result<(), AuthError> {
        self.iter_mut()
            .find(|old| old.uid == user.uid)
            .map(|old| *old = user)
            .ok_or(AuthError::UserNotFound)
    }

    fn add_user(&mut self, user: User) -> Result<(), AuthError> {
        if self.get_user_by_uid(&user.uid).is_some() {
            return Err(AuthError::UserAlreadyExists);
        }

        self.push(user);

        Ok(())
    }

    fn remove_user(&mut self, uid: impl AsRef<str>) -> Result<(), AuthError> {
        if self.get_user_by_uid(&uid).is_none() {
            return Err(AuthError::UserNotFound);
        }

        self.retain(|user| user.uid != uid.as_ref());

        Ok(())
    }
}
