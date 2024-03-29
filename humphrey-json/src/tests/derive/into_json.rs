use crate::prelude::*;

#[test]
fn struct_into_json() {
    #[derive(IntoJson)]
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
        test_1.to_json(),
        json!({
            "a": "some value",
            "b": false,
            "c": 123
        })
    );

    assert_eq!(
        test_2.to_json(),
        json!({
            "a": "some other value",
            "b": true,
            "c": null
        })
    );
}

#[test]
fn nested_structs_into_json() {
    #[derive(IntoJson)]
    struct Person {
        name: String,
        location: Location,
    }

    #[derive(IntoJson)]
    struct Location {
        city: String,
        country: String,
        lat: f64,
        lon: f64,
    }

    let test_person_1 = Person {
        name: "William Henderson".to_string(),
        location: Location {
            city: "London".to_string(),
            country: "United Kingdom".to_string(),
            lat: 51.5,
            lon: -0.1,
        },
    };

    let test_person_2 = Person {
        name: "Charles Hungus".to_string(),
        location: Location {
            city: "New York".to_string(),
            country: "United States".to_string(),
            lat: 40.7,
            lon: -74.0,
        },
    };

    assert_eq!(
        test_person_1.to_json(),
        json!({
            "name": "William Henderson",
            "location": {
                "city": "London",
                "country": "United Kingdom",
                "lat": 51.5,
                "lon": (-0.1)
            }
        })
    );

    assert_eq!(
        test_person_2.to_json(),
        json!({
            "name": "Charles Hungus",
            "location": {
                "city": "New York",
                "country": "United States",
                "lat": 40.7,
                "lon": (-74.0)
            }
        })
    );
}

#[test]
fn renamed_struct_into_json() {
    #[derive(IntoJson)]
    struct Test {
        a: String,
        b: bool,
        #[rename = "d"]
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
        test_1.to_json(),
        json!({
            "a": "some value",
            "b": false,
            "d": 123
        })
    );

    assert_eq!(
        test_2.to_json(),
        json!({
            "a": "some other value",
            "b": true,
            "d": null
        })
    );
}

#[test]
fn tuple_struct_into_json() {
    #[derive(IntoJson)]
    struct TupleStruct(String, i32);

    assert_eq!(
        TupleStruct("some value".to_string(), 123).to_json(),
        json!(["some value", 123])
    );

    assert_eq!(
        TupleStruct("some other value".to_string(), 456).to_json(),
        json!(["some other value", 456])
    );
}

#[test]
fn enum_into_json() {
    #[derive(IntoJson)]
    enum Enum {
        VariantA,
        VariantB,
        VariantC,
    }

    assert_eq!(Enum::VariantA.to_json(), json!("VariantA"));
    assert_eq!(Enum::VariantB.to_json(), json!("VariantB"));
    assert_eq!(Enum::VariantC.to_json(), json!("VariantC"));
}

#[test]
fn renamed_enum_into_json() {
    #[derive(IntoJson, PartialEq, Debug)]
    enum Enum {
        #[rename = "variant_a"]
        VariantA,
        #[rename = "variant_b"]
        VariantB,
        #[rename = "variant_c"]
        VariantC,
    }

    assert_eq!(Enum::VariantA.to_json(), json!("variant_a"));
    assert_eq!(Enum::VariantB.to_json(), json!("variant_b"));
    assert_eq!(Enum::VariantC.to_json(), json!("variant_c"));
}
