pub mod database;
pub mod error;
pub mod user;

use crate::database::AuthDatabase;
use crate::error::AuthError;
pub use crate::user::User;

#[derive(Default)]
pub struct AuthProvider<T>
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

    pub fn create(&mut self, password: impl AsRef<str>) -> Result<User, AuthError> {
        let new_user = User::create(password)?;
        self.users.add_user(new_user.clone())?;

        Ok(new_user)
    }

    pub fn exists(&mut self, uid: impl AsRef<str>) -> bool {
        self.users.get_user_by_uid(&uid).is_some()
    }

    pub fn verify<S>(&self, uid: S, password: S) -> bool
    where
        S: AsRef<str>,
    {
        self.users
            .get_user_by_uid(&uid)
            .map(|user| user.verify(&password))
            .unwrap_or(false)
    }

    pub fn remove(&mut self, uid: impl AsRef<str>) -> Result<(), AuthError> {
        self.users.remove_user(&uid)
    }
}
