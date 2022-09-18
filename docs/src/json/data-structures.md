# Strongly-Typed Data Structures
Humphrey JSON provides a powerful way of using Rust data structures to work with JSON data. Mappings between JSON and Rust types can be automatically generated using the `FromJson` and `IntoJson` derive macro, as well as configured more explicitly using the `json_map!` macro.

## Deriving `FromJson` and `IntoJson`
The derive macros can only be used when the `derive` feature is enabled, which it is by default. The `FromJson` and `IntoJson` traits can be derived for a type as follows.

```rs
use humphrey_json::prelude::*;

#[derive(FromJson, IntoJson)]
struct User {
    name: String,
    location: String,
}
```

The macros also support tuple structs and basic enums, but do not yet support enums with variants that have fields. Every type contained within the struct must already implement the traits that are being implemented on the struct.

```rs
#[derive(FromJson, IntoJson)]
struct TupleStruct(String, u8);

#[derive(FromJson, IntoJson)]
enum MyEnum {
    Yes,
    No,
    Maybe,
}
```

Finally, the macros also provide a `rename` attribute, which can be used to rename the fields of a struct or the variants of an enum in the JSON data.

```rs
#[derive(FromJson, IntoJson)]
struct RenamedFields {
    #[rename = "dateOfBirth"]
    date_of_birth: String,
}

#[derive(FromJson, IntoJson)]
enum RenamedVariants {
    #[rename = "y"]
    Yes,
    #[rename = "n"]
    No,
    #[rename = "?"]
    Maybe,
}
```

## The `json_map!` Macro
The `json_map!` macro is used as follows. The fields on the left represent the fields of the struct, and there must be an entry for each field in the struct. The strings on the right represent the names of the fields in the JSON data. It automatically generates a `FromJson` and `IntoJson` implementation for the struct.

Unlike the derive macros, this macro allows you to specify exactly what names you want to use for each field, instead of just using the struct's field names. On the downside, however, you cannot use the `json_map!` macro on enums.

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

This also works for more complex structs, provided that all nested types implement `FromJson`.

## Serializing into JSON
Instances of any struct which implements `IntoJson` can be serialized into JSON, as follows:

```rs
let json_string = humphrey_json::to_string(&user).unwrap();
```

To format the JSON with newlines and to customize the indentation, you can use the `to_string_pretty` function, which takes the number of spaces to indent as an argument.

```rs
let json_string = humphrey_json::to_string_pretty(&user).unwrap();
```

## Conclusion
In conclusion, the derive macros, the `json_map!` macro and their associated functions are a powerful way of working with typed JSON data. To find out more about Humphrey JSON, consider looking at the [API reference](https://docs.rs/humphrey-json).