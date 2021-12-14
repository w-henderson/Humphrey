#[derive(Debug)]
pub enum AuthError {
    GenericError,
    UserNotFound,
    UserAlreadyExists,
}
