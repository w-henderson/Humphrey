use crate::{AuthProvider, User};

#[test]
fn integration_test() {
    let mut provider: AuthProvider<Vec<User>> = AuthProvider::default();

    let uid = provider.create_user("hunter42").unwrap();

    assert!(provider.exists(&uid));
    assert!(provider.verify(&uid, "hunter42"));
    assert!(!provider.verify(&uid, "hunter43"));

    provider.remove_user(&uid).unwrap();

    assert!(!provider.exists(&uid));
}
