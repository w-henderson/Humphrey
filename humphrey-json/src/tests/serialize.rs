use crate::Value;

#[test]
fn serialize_object() {
    let string = include_str!("./testcases/object.json");
    let value = Value::parse(string).unwrap();

    let serialized = value.serialize();
    let expected = "{\"name\":\"William Henderson\",\"favouriteNumber\":1.414,\"languages\":[\"Rust\",\"TypeScript\",1234,false,{\"name\":\"Python\",\"version\":3.6}],\"weaknesses\":[],\"funnyName\":{},\"online\":true,\"life\":null}".to_string();

    assert_eq!(serialized, expected);
}

#[test]
fn serialize_array() {
    let string = include_str!("./testcases/array.json");
    let value = Value::parse(string).unwrap();

    let serialized = value.serialize();
    let expected = "[{\"name\":\"w-henderson\",\"favouriteNumber\":1.414,\"online\":true},{\"name\":\"flauntingspade4\",\"favouriteNumber\":69,\"online\":false}]";

    assert_eq!(serialized, expected);
}

#[test]
fn serialize_literals() {
    let strings = ["1.414", "true", "null"];
    let values: Vec<Value> = strings.iter().map(|&s| Value::parse(s).unwrap()).collect();

    let serialized: Vec<String> = values.iter().map(|v| v.serialize()).collect();
    let expected = strings;

    assert_eq!(serialized, expected);
}

#[test]
fn serialize_utf8_escape() {
    let value = Value::String("\0\n".into());

    let serialized = value.serialize();
    let expected = "\"\\u0000\\n\"";

    assert_eq!(serialized, expected);
}

#[test]
fn serialize_object_pretty() {
    let string = include_str!("./testcases/object.json");
    let value = Value::parse(string).unwrap();

    let serialized = value.serialize_pretty(4);
    let expected = r#"{
    "name": "William Henderson",
    "favouriteNumber": 1.414,
    "languages": [
        "Rust",
        "TypeScript",
        1234,
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
}"#
    .to_string();

    assert_eq!(serialized, expected);
}

#[test]
fn serialize_array_pretty() {
    let string = include_str!("./testcases/array.json");
    let value = Value::parse(string).unwrap();

    let serialized = value.serialize_pretty(4);
    let expected = r#"[
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
]"#;

    assert_eq!(serialized, expected);
}

#[test]
fn serialize_object_pretty_custom_indent() {
    let string = include_str!("./testcases/object.json");
    let value = Value::parse(string).unwrap();

    let serialized = value.serialize_pretty(2);
    let expected = r#"{
  "name": "William Henderson",
  "favouriteNumber": 1.414,
  "languages": [
    "Rust",
    "TypeScript",
    1234,
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
}"#
    .to_string();

    assert_eq!(serialized, expected);
}

#[test]
fn serialize_array_pretty_custom_indent() {
    let string = include_str!("./testcases/array.json");
    let value = Value::parse(string).unwrap();

    let serialized = value.serialize_pretty(2);
    let expected = r#"[
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
]"#;

    assert_eq!(serialized, expected);
}
