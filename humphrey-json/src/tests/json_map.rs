use crate::prelude::*;

#[derive(PartialEq, Debug)]
struct Person {
    name: String,
    favourite_number: f64,
    online: bool,
}

#[derive(PartialEq, Debug)]
struct GroupOfPeople {
    best_person: Option<Person>,
    people: Vec<Person>,
}

json_map! {
    Person,
    name => "name",
    favourite_number => "favouriteNumber",
    online => "online"
}

json_map! {
    GroupOfPeople,
    best_person => "bestPerson",
    people => "people"
}

#[test]
fn test_from_json() {
    let value = json!({
        "name": "William Henderson",
        "favouriteNumber": 1.414,
        "online": true
    });

    let person = Person::from_json(&value).unwrap();

    assert_eq!(person.name, "William Henderson");
    assert_eq!(person.favourite_number, 1.414);
    assert!(person.online);
}

#[test]
fn test_into_json() {
    let person = Person {
        name: "William Henderson".into(),
        favourite_number: 1.414,
        online: true,
    };

    let value = person.to_json();

    assert_eq!(
        value,
        json!({
            "name": "William Henderson",
            "favouriteNumber": 1.414,
            "online": true
        })
    );
}

#[test]
fn test_nested_from_json() {
    let value = json!({
        "bestPerson": {
            "name": "w-henderson",
            "favouriteNumber": 1.414,
            "online": true
        },
        "people": [
            {
                "name": "w-henderson",
                "favouriteNumber": 1.414,
                "online": true
            },
            {
                "name": "flauntingspade4",
                "favouriteNumber": 2.72,
                "online": false
            }
        ]
    });

    let group_of_people = GroupOfPeople::from_json(&value).unwrap();

    assert_eq!(
        group_of_people,
        GroupOfPeople {
            best_person: Some(Person {
                name: "w-henderson".into(),
                favourite_number: 1.414,
                online: true,
            }),
            people: vec![
                Person {
                    name: "w-henderson".into(),
                    favourite_number: 1.414,
                    online: true,
                },
                Person {
                    name: "flauntingspade4".into(),
                    favourite_number: 2.72,
                    online: false,
                },
            ],
        }
    );
}

#[test]
fn test_nested_into_json() {
    let group_of_people = GroupOfPeople {
        best_person: Some(Person {
            name: "w-henderson".into(),
            favourite_number: 1.414,
            online: true,
        }),
        people: vec![
            Person {
                name: "w-henderson".into(),
                favourite_number: 1.414,
                online: true,
            },
            Person {
                name: "flauntingspade4".into(),
                favourite_number: 2.72,
                online: false,
            },
        ],
    };

    let value = group_of_people.to_json();

    assert_eq!(
        value,
        json!({
            "bestPerson": {
                "name": "w-henderson",
                "favouriteNumber": 1.414,
                "online": true
            },
            "people": [
                {
                    "name": "w-henderson",
                    "favouriteNumber": 1.414,
                    "online": true
                },
                {
                    "name": "flauntingspade4",
                    "favouriteNumber": 2.72,
                    "online": false
                }
            ]
        })
    );
}

#[test]
fn test_optional_missing_from_json() {
    let value = json!({
        "people": [
            {
                "name": "w-henderson",
                "favouriteNumber": 1.414,
                "online": true
            },
            {
                "name": "flauntingspade4",
                "favouriteNumber": 2.72,
                "online": false
            }
        ]
    });

    let group_of_people = GroupOfPeople::from_json(&value).unwrap();

    assert_eq!(
        group_of_people,
        GroupOfPeople {
            best_person: None,
            people: vec![
                Person {
                    name: "w-henderson".into(),
                    favourite_number: 1.414,
                    online: true,
                },
                Person {
                    name: "flauntingspade4".into(),
                    favourite_number: 2.72,
                    online: false,
                },
            ],
        }
    );
}

#[test]
fn test_optional_missing_into_json() {
    let group_of_people = GroupOfPeople {
        best_person: None,
        people: vec![
            Person {
                name: "w-henderson".into(),
                favourite_number: 1.414,
                online: true,
            },
            Person {
                name: "flauntingspade4".into(),
                favourite_number: 2.72,
                online: false,
            },
        ],
    };

    let value = group_of_people.to_json();

    assert_eq!(
        value,
        json!({
            "bestPerson": null,
            "people": [
                {
                    "name": "w-henderson",
                    "favouriteNumber": 1.414,
                    "online": true
                },
                {
                    "name": "flauntingspade4",
                    "favouriteNumber": 2.72,
                    "online": false
                }
            ]
        })
    );
}
