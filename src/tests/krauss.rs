#[allow(unused_imports)]
use crate::krauss::wildcard_match;

#[test]
fn test_match() {
    // Identical matches
    assert!(wildcard_match("ab", "ab"));
    assert!(!wildcard_match("ab", "cd"));

    // Zero character matches
    assert!(wildcard_match("ab*", "ab"));
    assert!(wildcard_match("*ab", "ab"));

    // Basic matches
    assert!(wildcard_match("ab*", "abcd"));
    assert!(wildcard_match("*cd", "abcd"));
    assert!(!wildcard_match("ab", "abcd"));

    // Multiple matches
    assert!(wildcard_match("ab*ef", "abcdef"));
    assert!(wildcard_match("ab*d", "abcd"));
    assert!(wildcard_match("ab*ef", "abef"));
    assert!(wildcard_match("*cd*", "cd"));
    assert!(wildcard_match("*cd*", "abcd"));
    assert!(wildcard_match("*cd*", "cdef"));
    assert!(wildcard_match("*cd*", "abcdef"));

    // Just matches
    assert!(wildcard_match("*", "ab"));

    // Empty string
    assert!(wildcard_match("", ""));
    assert!(wildcard_match("*", ""));
}
