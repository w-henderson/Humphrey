use crate::error::AuthError;
use crate::{AuthProvider, User};

use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn session_test() -> Result<(), Box<dyn Error>> {
    let mut auth: AuthProvider<Vec<User>> = AuthProvider::default();

    let uid_1 = auth.create_user("password1")?;
    let uid_2 = auth.create_user("password2")?;

    let token_1 = auth.create_session_with_lifetime(&uid_1, 1)?;
    let token_2 = auth.create_session(&uid_2)?;

    let err = Err(AuthError::InvalidToken);

    // Both tokens are valid
    assert_eq!(auth.get_uid_by_token(&token_1), Ok(uid_1.clone()));
    assert_eq!(auth.get_uid_by_token(&token_2), Ok(uid_2.clone()));

    auth.invalidate_session(&token_2);

    // Token 1 is still valid but token 2 has been revoked
    assert_eq!(auth.get_uid_by_token(&token_1), Ok(uid_1.clone()));
    assert_eq!(auth.get_uid_by_token(&token_2), err);

    // Wait for token 1 to expire
    sleep(Duration::from_secs(1));

    // Both tokens are invalid
    assert_eq!(auth.get_uid_by_token(&token_1), err);
    assert_eq!(auth.get_uid_by_token(&token_2), err);

    Ok(())
}
