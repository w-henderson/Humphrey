use humphrey_auth::database::AuthDatabase as AuthDatabaseTrait;
use humphrey_auth::error::AuthError;
use humphrey_auth::session::Session;
use humphrey_auth::user::User;

use std::sync::Mutex;

use jasondb::query;
use jasondb::Database;

pub struct WrappedDatabase(pub Mutex<Database<User>>);

impl WrappedDatabase {
    pub fn new(db: Database<User>) -> Self {
        Self(Mutex::new(db))
    }
}

impl AuthDatabaseTrait for WrappedDatabase {
    fn get_user_by_uid(&self, uid: impl AsRef<str>) -> Option<User> {
        let mut db = self.0.lock().ok()?;

        db.get(uid).ok()
    }

    fn get_user_by_token(&self, token: impl AsRef<str>) -> Option<User> {
        let mut db = self.0.lock().ok()?;
        let token = token.as_ref();

        db.query(query!(session.token == token))
            .ok()?
            .next()?
            .ok()
            .map(|(_, user)| user)
    }

    fn get_session_by_token(&self, token: impl AsRef<str>) -> Option<Session> {
        self.get_user_by_token(token)
            .map(|user| user.session.unwrap())
    }

    fn update_user(&mut self, user: User) -> Result<(), AuthError> {
        let mut db = self.0.lock().map_err(|_| AuthError::GenericError)?;
        let uid = user.uid.clone();

        db.set(uid, user).map_err(|_| AuthError::GenericError)
    }

    fn add_user(&mut self, user: User) -> Result<(), AuthError> {
        self.update_user(user)
    }

    fn remove_user(&mut self, uid: impl AsRef<str>) -> Result<(), AuthError> {
        let mut db = self.0.lock().map_err(|_| AuthError::GenericError)?;

        db.delete(uid).map_err(|_| AuthError::GenericError)
    }
}
