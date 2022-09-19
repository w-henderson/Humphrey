use crate::{json, Value};

#[test]
fn test_macro_object() {
    let value = json!({
        "name": "William Henderson",
        "favouriteNumber": 1.414,
        "languages": [
            "Rust",
            "TypeScript",
            1.234E3,
            false,
            {
                "name": "Python",
                "version": 3.6
            }
        ],
        "weaknesses": [],
        "funnyName": {},
        "online": true,
        "life": null
    });

    let expected_string = include_str!("./testcases/object.json");
    let expected_value = Value::parse(expected_string).unwrap();

    assert_eq!(value, expected_value);
}

#[test]
fn test_macro_array() {
    let value = json!([
      {
        "name": "w-henderson",
        "favouriteNumber": 1.414,
        "online": true
      },
      {
        "name": "flauntingspade4",
        "favouriteNumber": 69,
        "online": false
      }
    ]);

    let expected_string = include_str!("./testcases/array.json");
    let expected_value = Value::parse(expected_string).unwrap();

    assert_eq!(value, expected_value);
}

#[test]
fn test_macro_string() {
    let value = json!("Hello, world!");
    let expected_value = Value::String("Hello, world!".into());

    assert_eq!(value, expected_value);
}

#[test]
fn test_macro_literals() {
    let values = [json!(1234), json!(true), json!(false), json!(null), json!()];

    let expected_values = [
        Value::Number(1234.0),
        Value::Bool(true),
        Value::Bool(false),
        Value::Null,
        Value::Null,
    ];

    for (value, expected_value) in values.iter().zip(expected_values.iter()) {
        assert_eq!(*value, *expected_value);
    }
}

#[test]
fn test_macro_embedded_variables() {
    let embedded_string = "Hello, world!";
    let embedded_number: u16 = 1234;
    let embedded_some = Some("value");
    let embedded_none: Option<&str> = None;

    let value = json!({
        "string": embedded_string,
        "number": embedded_number,
        "some": embedded_some,
        "none": embedded_none
    });

    let expected_value = Value::Object(vec![
        ("string".into(), Value::String("Hello, world!".into())),
        ("number".into(), Value::Number(1234.0)),
        ("some".into(), Value::String("value".into())),
        ("none".into(), Value::Null),
    ]);

    assert_eq!(value, expected_value);
}

#[test]
fn test_macro_complex_embedded_variables() {
    let embedded_string = "Hello, world!";
    let embedded_number: Option<u16> = Some(1234);

    let value = json!({
        "string": embedded_string,
        "number": embedded_number.map(|n| n * 2)
    });

    let expected_value = Value::Object(vec![
        ("string".into(), Value::String("Hello, world!".into())),
        ("number".into(), Value::Number(2468.0)),
    ]);

    assert_eq!(value, expected_value);
}
