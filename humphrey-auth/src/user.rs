use crate::error::AuthError;
use crate::session::Session;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

use uuid::Uuid;

#[derive(Clone)]
pub struct User {
    pub uid: String,
    pub session: Option<Session>,
    password_hash: String,
}

impl User {
    pub fn create(password: impl AsRef<str>) -> Result<User, AuthError> {
        let uid = Uuid::new_v4().to_string();
        let password = password.as_ref();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::GenericError)?
            .to_string();

        Ok(Self {
            uid,
            session: None,
            password_hash,
        })
    }

    pub fn verify(&self, password: impl AsRef<str>) -> bool {
        let password = password.as_ref().as_bytes();
        let argon2 = Argon2::default();
        let password_hash = PasswordHash::new(self.password_hash.as_str()).unwrap();

        argon2.verify_password(password, &password_hash).is_ok()
    }
}

impl AsRef<str> for User {
    fn as_ref(&self) -> &str {
        self.uid.as_ref()
    }
}
