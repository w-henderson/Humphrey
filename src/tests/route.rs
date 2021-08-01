#[allow(unused_imports)]
use crate::route::Uri;

#[test]
fn test_uri_from_string() {
    let expected_index = Uri::new(Vec::new(), None);
    let expected_simple = Uri::new(vec!["contact".to_string()], None);
    let expected_double = Uri::new(
        vec!["exampleblog".to_string(), "introduction".to_string()],
        None,
    );
    let expected_query = Uri::new(
        vec!["api".to_string(), "send".to_string()],
        Some("data=hello".to_string()),
    );

    let index: Uri = "/".parse().unwrap();
    let simple: Uri = "/contact".parse().unwrap();
    let double: Uri = "/exampleblog/introduction".parse().unwrap();
    let query: Uri = "/api/send?data=hello".parse().unwrap();
    let error = "this is not a valid uri".parse::<Uri>();

    assert_eq!(index, expected_index);
    assert_eq!(simple, expected_simple);
    assert_eq!(double, expected_double);
    assert_eq!(query, expected_query);
    assert!(error.is_err());
}
