use crate::Value;

#[test]
fn test_index_object() {
    let string = include_str!("./testcases/object.json");
    let value = Value::parse(string).unwrap();

    assert_eq!(value["name"], Value::String("William Henderson".into()));
    assert_eq!(value["favouriteNumber"], Value::Number(1.414));
    assert_eq!(value["languages"][0], Value::String("Rust".into()));
    assert_eq!(
        value["languages"][4]["name"],
        Value::String("Python".into())
    );
}

#[test]
fn test_index_array() {
    let string = include_str!("./testcases/array.json");
    let value = Value::parse(string).unwrap();

    assert_eq!(value[0]["name"], Value::String("w-henderson".into()));
    assert_eq!(value[1]["name"], Value::String("flauntingspade4".into()));

    assert_eq!(
        value[0],
        Value::Object(vec![
            ("name".into(), Value::String("w-henderson".into())),
            ("favouriteNumber".into(), Value::Number(1.414)),
            ("online".into(), Value::Bool(true)),
        ])
    );
}

#[test]
#[should_panic]
fn test_invalid_index() {
    let string = "\"Hello, world!\"";
    let value = Value::parse(string).unwrap();

    let _ = value["invalid"];
}
