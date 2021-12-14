use crate::error::AuthError;
use crate::user::User;

pub trait AuthDatabase {
    fn get_user_by_uid(&self, uid: impl AsRef<str>) -> Option<User>;
    fn add_user(&mut self, user: User) -> Result<(), AuthError>;
    fn remove_user(&mut self, uid: impl AsRef<str>) -> Result<(), AuthError>;
}

impl AuthDatabase for Vec<User> {
    fn get_user_by_uid(&self, uid: impl AsRef<str>) -> Option<User> {
        self.iter()
            .find(|user| user.uid == uid.as_ref())
            .map(|user| (*user).clone())
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
