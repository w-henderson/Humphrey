use crate::error::AuthError;
use crate::session::Session;
use crate::user::User;

pub trait AuthDatabase {
    fn get_user_by_uid(&self, uid: impl AsRef<str>) -> Option<User>;
    fn get_user_by_token(&self, token: impl AsRef<str>) -> Option<User>;
    fn get_session_by_token(&self, token: impl AsRef<str>) -> Option<Session>;

    fn update_user(&mut self, user: User) -> Result<(), AuthError>;
    fn add_user(&mut self, user: User) -> Result<(), AuthError>;
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
