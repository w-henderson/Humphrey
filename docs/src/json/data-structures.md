# Strongly-Typed Data Structures
Humphrey JSON provides a powerful way of using Rust data structures to work with JSON data. Mappings between JSON and Rust types are defined using the `json_map!` macro.

## The `json_map!` Macro
The macro is used as follows. The fields on the left represent the fields of the struct, and there must be an entry for each field in the struct. The strings on the right represent the names of the fields in the JSON data. It automatically generates a `FromJson` and `IntoJson` implementation for the struct.

```rs
use humphrey_json::prelude::*;

#[derive(PartialEq, Eq)] // not required, but used later in this section
struct User {
    name: String,
    location: String,
}

json_map! {
    User,
    name => "name",
    location => "country"
}
```

## Parsing into a Struct
To parse a JSON string into a struct, you can simply use the `humphrey_json::from_str` function. For example, given the following JSON data, you can parse it into a `User` struct as follows:

```json
{
    "name": "Humphrey",
    "country": "United Kingdom"
}
```

```rs
let user: User = humphrey_json::from_str(json_string).unwrap();

assert_eq!(user, User {
    name: "Humphrey".to_string(),
    location: "United Kingdom".to_string()
});
```

This also works for more complex structs, provided that all nested types implement `FromJson`. Currently, the macro cannot be used for enums so `FromJson` must be implemented manually for these types.

## Serializing into JSON
Instances of any struct which implements `IntoJson` can be serialized into JSON, as follows:

```rs
let json_string = humphrey_json::to_string(&user).unwrap();
```

## Conclusion
In conclusion, the `json_map!` macro and its associated functions are a powerful way of working with typed JSON data. To find out more about Humphrey JSON, consider looking at the [API reference](https://docs.rs/humphrey-json).