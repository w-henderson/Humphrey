//! Provides a user model for the authentication service.

use crate::error::AuthError;
use crate::session::Session;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};

use uuid::Uuid;

/// Represents a user.
#[derive(Clone)]
pub struct User {
    /// The unique ID of the user.
    pub uid: String,
    /// The user's current session, if they have one.
    pub session: Option<Session>,
    /// The Argon2 hashed password of the user.
    pub password_hash: String,
}

impl User {
    /// Creates a user with the given password.
    /// Returns the user object of the new user.
    pub fn create(password: impl AsRef<str>, pepper: Option<&[u8]>) -> Result<User, AuthError> {
        let uid = Uuid::new_v4().to_string();
        let password = password.as_ref();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = create_argon2_instance(pepper);

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

    /// Verifies that the given password matches the password of the user.
    pub fn verify(&self, password: impl AsRef<str>, pepper: Option<&[u8]>) -> bool {
        let password = password.as_ref().as_bytes();
        let argon2 = create_argon2_instance(pepper);

        let password_hash = PasswordHash::new(self.password_hash.as_str()).unwrap();

        argon2.verify_password(password, &password_hash).is_ok()
    }
}

fn create_argon2_instance(pepper: Option<&[u8]>) -> Argon2<'_> {
    pepper
        .map(|pepper| {
            Argon2::new_with_secret(
                pepper,
                Algorithm::default(),
                Version::default(),
                Params::default(),
            )
            .unwrap()
        })
        .unwrap_or_default()
}

impl AsRef<str> for User {
    fn as_ref(&self) -> &str {
        self.uid.as_ref()
    }
}
