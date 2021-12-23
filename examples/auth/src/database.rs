use humphrey_auth::database::AuthDatabase as AuthDatabaseTrait;
use humphrey_auth::error::AuthError;
use humphrey_auth::session::Session;
use humphrey_auth::user::User;

use std::sync::Arc;

use jasondb::database::Document;
use jasondb::JasonDB;

#[derive(Clone)]
pub struct WrappedDatabase(pub Arc<JasonDB>);

impl WrappedDatabase {
    pub fn new(db: JasonDB) -> Self {
        Self(Arc::new(db))
    }
}

impl AuthDatabaseTrait for WrappedDatabase {
    fn get_user_by_uid(&self, uid: impl AsRef<str>) -> Option<User> {
        let db = self.0.read();
        let auth = db.collection("auth").unwrap();
        let user = auth.get(uid);

        user.map(deserialize_user)
    }

    fn get_user_by_token(&self, token: impl AsRef<str>) -> Option<User> {
        let db = self.0.read();
        let auth = db.collection("auth").unwrap();
        let user = auth.list().iter().find(|user| {
            let user = deserialize_user(user);
            user.session
                .map(|s| s.token == token.as_ref())
                .unwrap_or(false)
        });

        user.map(deserialize_user)
    }

    fn get_session_by_token(&self, token: impl AsRef<str>) -> Option<Session> {
        self.get_user_by_token(token)
            .map(|user| user.session.unwrap())
    }

    fn update_user(&mut self, user: User) -> Result<(), AuthError> {
        let mut db = self.0.write();
        let auth = db.collection_mut("auth").unwrap();

        if auth.set(user.uid.as_str(), serialize_user(user.clone())) {
            Ok(())
        } else {
            Err(AuthError::GenericError)
        }
    }

    fn add_user(&mut self, user: User) -> Result<(), AuthError> {
        self.update_user(user)
    }

    fn remove_user(&mut self, uid: impl AsRef<str>) -> Result<(), AuthError> {
        let mut db = self.0.write();
        let auth = db.collection_mut("auth").unwrap();

        if auth.remove(uid.as_ref()) {
            Ok(())
        } else {
            Err(AuthError::GenericError)
        }
    }
}

fn serialize_user(user: User) -> String {
    if let Some(session) = user.session {
        format!(
            "{};{};{};{}",
            user.uid, user.password_hash, session.token, session.expiry
        )
    } else {
        format!("{};{}", user.uid, user.password_hash)
    }
}

fn deserialize_user(document: &Document) -> User {
    let mut split = document.json.split(';');
    let uid = split.next().unwrap();
    let hash = split.next().unwrap();

    let session = if let Some(token) = split.next() {
        let expiry: u64 = split.next().unwrap().parse().unwrap();

        Some(Session {
            token: token.to_string(),
            expiry,
        })
    } else {
        None
    };

    User {
        uid: uid.to_string(),
        password_hash: hash.to_string(),
        session,
    }
}
