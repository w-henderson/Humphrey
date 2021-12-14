use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    GenericError,
    UserNotFound,
    UserAlreadyExists,
    InvalidToken,
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
