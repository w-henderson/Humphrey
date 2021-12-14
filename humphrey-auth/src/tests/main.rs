use crate::{AuthProvider, User};

#[test]
fn integration_test() {
    let mut provider: AuthProvider<Vec<User>> = AuthProvider::default();

    let user = provider.create("hunter42").unwrap();

    assert!(provider.exists(&user.uid));
    assert!(provider.verify(user.uid.as_str(), "hunter42"));
    assert!(!provider.verify(user.uid.as_str(), "hunter43"));

    provider.remove(user.uid.as_str()).unwrap();

    assert!(!provider.exists(&user.uid));
}
