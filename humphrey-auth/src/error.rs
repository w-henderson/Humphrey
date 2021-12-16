use std::error::Error;
use std::fmt::Display;

/// Represents an error with authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// An unknown error.
    GenericError,
    /// The given user could not be found.
    UserNotFound,
    /// The given user already exists.
    UserAlreadyExists,
    /// The given token does not exist or has expired.
    InvalidToken,
    /// A session for the given user already exists.
    SessionAlreadyExists,
}

impl Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::GenericError => write!(f, "Unknown error"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::UserAlreadyExists => write!(f, "User already exists"),
            AuthError::InvalidToken => write!(f, "Invalid token"),
            AuthError::SessionAlreadyExists => write!(f, "Session already exists"),
        }
    }
}

impl Error for AuthError {}
