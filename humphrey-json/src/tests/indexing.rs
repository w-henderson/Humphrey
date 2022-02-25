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

    let mut value_mut = value;

    value_mut["name"] = json!("Humphrey");
    value_mut["favouriteNumber"] = json!(1.2E1);

    assert_eq!(value_mut["name"], Value::String("Humphrey".into()));
    assert_eq!(value_mut["favouriteNumber"], Value::Number(12.0));
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

    let mut value_mut = value;

    value_mut[0]["online"] = json!(false);

    assert_eq!(
        value_mut[0],
        Value::Object(vec![
            ("name".into(), Value::String("w-henderson".into())),
            ("favouriteNumber".into(), Value::Number(1.414)),
            ("online".into(), Value::Bool(false)),
        ])
    );
}

#[test]
fn test_invalid_index() {
    let string = "\"Hello, world!\"";
    let value = Value::parse(string).unwrap();

    assert_eq!(value["name"], Value::Null);
}

#[test]
fn test_add_index() {
    let string = "{}";
    let mut value = Value::parse(string).unwrap();

    value["name"] = json!("Humphrey");

    assert_eq!(
        value,
        json!({
            "name": "Humphrey"
        })
    );
}

#[test]
#[should_panic]
fn test_add_invalid_index() {
    let string = "\"Hello, world!\"";
    let mut value = Value::parse(string).unwrap();

    value["name"] = json!("Humphrey");
}
