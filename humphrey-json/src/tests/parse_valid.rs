use crate::Value;

#[test]
fn test_simple_object() {
    let string = r#"{"asd":"sdf", "dfg":"fgh"}"#;
    let value = Value::parse(string).unwrap();

    let expected_value = Value::Object(vec![
        ("asd".into(), Value::String("sdf".into())),
        ("dfg".into(), Value::String("fgh".into())),
    ]);

    assert_eq!(value, expected_value);
}

#[test]
fn test_object() {
    let string = include_str!("./testcases/object.json");
    let value = Value::parse(string).unwrap();

    let expected_value = Value::Object(vec![
        ("name".into(), Value::String("William Henderson".into())),
        ("favouriteNumber".into(), Value::Number(1.414)),
        (
            "languages".into(),
            Value::Array(vec![
                Value::String("Rust".into()),
                Value::String("TypeScript".into()),
                Value::Number(1234.0),
                Value::Bool(false),
                Value::Object(vec![
                    ("name".into(), Value::String("Python".into())),
                    ("version".into(), Value::Number(3.6)),
                ]),
            ]),
        ),
        ("weaknesses".into(), Value::Array(vec![])),
        ("funnyName".into(), Value::Object(vec![])),
        ("online".into(), Value::Bool(true)),
        ("life".into(), Value::Null),
    ]);

    assert_eq!(value, expected_value);
}

#[test]
fn test_array() {
    let string = include_str!("./testcases/array.json");
    let value = Value::parse(string).unwrap();

    let expected_value = Value::Array(vec![
        Value::Object(vec![
            ("name".into(), Value::String("w-henderson".into())),
            ("favouriteNumber".into(), Value::Number(1.414)),
            ("online".into(), Value::Bool(true)),
        ]),
        Value::Object(vec![
            ("name".into(), Value::String("flauntingspade4".into())),
            ("favouriteNumber".into(), Value::Number(69.0)),
            ("online".into(), Value::Bool(false)),
        ]),
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
        Value::Object(vec![]),
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

#[test]
fn test_four_byte_unicode_value() {
    let string = "\"\\ud83d\\ude02\"";
    let value = Value::parse(string).unwrap();

    let expected_value = Value::String("😂".into());
    assert_eq!(value, expected_value);
}
