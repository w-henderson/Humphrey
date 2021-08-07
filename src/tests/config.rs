#![allow(unused_imports)]
use crate::config;
use std::collections::HashMap;

#[test]
fn test_parse_ini() {
    let testcase =
        "[s1]\nkey1=value \nkey2 = value2   ; comment\nkey3 = \"value\"\n\n[s2]\nkey1=value";

    let mut expected_hashmap: HashMap<String, String> = HashMap::new();
    expected_hashmap.insert("s1.key1".into(), "value".into());
    expected_hashmap.insert("s1.key2".into(), "value2".into());
    expected_hashmap.insert("s1.key3".into(), "value".into());
    expected_hashmap.insert("s2.key1".into(), "value".into());

    let hashmap = config::parse_ini(testcase).unwrap();
    assert_eq!(hashmap, expected_hashmap);
}
