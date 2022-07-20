use crate::prelude::*;

#[test]
fn struct_from_json() {
    #[derive(FromJson, PartialEq, Debug)]
    struct Test {
        a: String,
        b: bool,
        c: Option<i32>,
    }

    let test_1 = Test {
        a: "some value".to_string(),
        b: false,
        c: Some(123),
    };

    let test_2 = Test {
        a: "some other value".to_string(),
        b: true,
        c: None,
    };

    assert_eq!(
        Test::from_json(&json!({
            "a": "some value",
            "b": false,
            "c": 123
        }))
        .unwrap(),
        test_1
    );

    assert_eq!(
        Test::from_json(&json!({
            "a": "some other value",
            "b": true,
            "c": null
        }))
        .unwrap(),
        test_2
    );
}

#[test]
fn nested_structs_from_json() {
    #[derive(FromJson, PartialEq, Debug)]
    struct Person {
        name: String,
        location: Location,
    }

    #[derive(FromJson, PartialEq, Debug)]
    struct Location {
        city: String,
        country: String,
        lat: f64,
        lon: f64,
    }

    assert_eq!(
        Person::from_json(&json!({
            "name": "William Henderson",
            "location": {
                "city": "London",
                "country": "United Kingdom",
                "lat": 51.5,
                "lon": (-0.1)
            }
        }))
        .unwrap(),
        Person {
            name: "William Henderson".to_string(),
            location: Location {
                city: "London".to_string(),
                country: "United Kingdom".to_string(),
                lat: 51.5,
                lon: -0.1,
            },
        }
    );

    assert_eq!(
        Person::from_json(&json!({
            "name": "Charles Hungus",
            "location": {
                "city": "New York",
                "country": "United States",
                "lat": 40.7,
                "lon": (-74.0)
            }
        }))
        .unwrap(),
        Person {
            name: "Charles Hungus".to_string(),
            location: Location {
                city: "New York".to_string(),
                country: "United States".to_string(),
                lat: 40.7,
                lon: -74.0,
            },
        }
    );
}

#[test]
fn tuple_struct_from_json() {
    #[derive(FromJson, PartialEq, Debug)]
    struct TupleStruct(String, i32);

    assert_eq!(
        TupleStruct::from_json(&json!(["some value", 123])).unwrap(),
        TupleStruct("some value".to_string(), 123)
    );

    assert_eq!(
        TupleStruct::from_json(&json!(["some other value", 456])).unwrap(),
        TupleStruct("some other value".to_string(), 456)
    );

    assert!(TupleStruct::from_json(&json!(["some value"])).is_err());
    assert!(TupleStruct::from_json(&json!([123, "some value"])).is_err());
    assert!(TupleStruct::from_json(&json!(["some value", 123, 456])).is_err());
}

#[test]
fn enum_from_json() {
    #[derive(FromJson, PartialEq, Debug)]
    enum Enum {
        VariantA,
        VariantB,
        VariantC,
    }

    assert_eq!(Enum::from_json(&json!("VariantA")).unwrap(), Enum::VariantA);
    assert_eq!(Enum::from_json(&json!("VariantB")).unwrap(), Enum::VariantB);
    assert_eq!(Enum::from_json(&json!("VariantC")).unwrap(), Enum::VariantC);

    assert!(Enum::from_json(&json!("VariantD")).is_err());
}
