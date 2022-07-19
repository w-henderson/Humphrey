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
