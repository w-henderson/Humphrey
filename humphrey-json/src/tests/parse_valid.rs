use crate::Value;

use super::hashmap_helper::to_object;

#[test]
fn test_simple_object() {
    let string = r#"{"asd":"sdf", "dfg":"fgh"}"#;
    let value = Value::parse(string).unwrap();

    let expected_value = Value::Object(to_object(vec![
        ("asd", Value::String("sdf".into())),
        ("dfg", Value::String("fgh".into())),
    ]));

    assert_eq!(value, expected_value);
}

#[test]
fn test_object() {
    let string = include_str!("./testcases/object.json");
    let value = Value::parse(string).unwrap();

    let expected_value = Value::Object(to_object(vec![
        ("name", Value::String("William Henderson".into())),
        ("favouriteNumber", Value::Number(1.414)),
        (
            "languages",
            Value::Array(vec![
                Value::String("Rust".into()),
                Value::String("TypeScript".into()),
                Value::Number(1234.0),
                Value::Bool(false),
                Value::Object(to_object(vec![
                    ("name", Value::String("Python".into())),
                    ("version", Value::Number(3.6)),
                ])),
            ]),
        ),
        ("weaknesses", Value::Array(vec![])),
        ("funnyName", Value::Object(to_object(vec![]))),
        ("online", Value::Bool(true)),
        ("life", Value::Null),
    ]));

    assert_eq!(value, expected_value);
}

#[test]
fn test_array() {
    let string = include_str!("./testcases/array.json");
    let value = Value::parse(string).unwrap();

    let expected_value = Value::Array(vec![
        Value::Object(to_object(vec![
            ("name", Value::String("w-henderson".into())),
            ("favouriteNumber", Value::Number(1.414)),
            ("online", Value::Bool(true)),
        ])),
        Value::Object(to_object(vec![
            ("name", Value::String("flauntingspade4".into())),
            ("favouriteNumber", Value::Number(69.0)),
            ("online", Value::Bool(false)),
        ])),
    ]);

    assert_eq!(value, expected_value);
}

#[test]
fn test_string() {
    let string = "\"Hello, World!\"";
    let value = Value::parse(string).unwrap();

    let expected_value = Value::String("Hello, World!".into());
    assert_eq!(value, expected_value);
}

#[test]
fn test_number() {
    let string = "1234.5678";
    let value = Value::parse(string).unwrap();

    let expected_value = Value::Number(1234.5678);
    assert_eq!(value, expected_value);
}

#[test]
fn test_null() {
    let string = "null";
    let value = Value::parse(string).unwrap();

    let expected_value = Value::Null;
    assert_eq!(value, expected_value);
}

#[test]
fn test_bool() {
    let (string_1, string_2) = ("true", "false");
    let (value_1, value_2) = (
        Value::parse(string_1).unwrap(),
        Value::parse(string_2).unwrap(),
    );

    let (expected_value_1, expected_value_2) = (Value::Bool(true), Value::Bool(false));

    assert_eq!(value_1, expected_value_1);
    assert_eq!(value_2, expected_value_2);
}

#[test]
fn test_whitespace() {
    let string = include_str!("./testcases/whitespace.json");
    let value = Value::parse(string).unwrap();

    let expected_value = Value::Array(vec![
        Value::String("whitespace".into()),
        Value::Number(1234.0),
        Value::Bool(true),
        Value::Object(to_object(vec![])),
    ]);

    assert_eq!(value, expected_value);
}

#[test]
fn test_escape_sequence() {
    let string = "\"H\\u0065llo, world\\u0021\\n\"";
    let value = Value::parse(string).unwrap();

    let expected_value = Value::String("Hello, world!\n".into());
    assert_eq!(value, expected_value);
}
