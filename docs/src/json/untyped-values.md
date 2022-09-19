# Untyped JSON Values

## Parsing Untyped JSON
JSON can be parsed into a `Value`, which can represent any JSON value. This can be done with either `humphrey_json::from_str` or `Value::parse`. Let's look at a simple example, which we'll use throughout the rest of this chapter.

```rs
use humphrey_json::Value;

fn main() {
    let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;

    let value: Value = humphrey_json::from_str(data).unwrap();

    println!("{:?}", value);
}
```

If you run this code, you'll see the internal representation of the parsed JSON value. The `Value` type must be specified, since the `from_str` function can return any type which implements `FromJson`, which we'll discuss later.

You can also use the `Value::parse` function like this:

```rs
let value = Value::parse(data).unwrap();
```

Now, we'll look at how to manipulate the JSON value.

## Manipulating JSON Values
Using the data from the previous example, we'll see how to access different fields of it.

You can index into the JSON value using the `get` and `get_mut` methods, which return `Option<&Value>` and `Option<&mut Value>` respectively. Alternatively, you can use Rust's indexing syntax (`value[index]`) where index is a number or a string, which returns `Value::Null` if the value does not exist and creates one if you attempt to set it.

You can extract the inner value of a JSON value using `as_bool`, `as_number`, `as_str`, `as_array` and `as_object`, all of which return options.

```rs
let name = value["name"].as_str();
let age = value.get("age").as_number();
let phone_1 = value["phones"].get(0);
let second_phone = value["phones"][1];

value["name"] = json!("Humphrey");
```

## Creating Untyped JSON
To create an untyped JSON value, you can use the `json!` macro. This allows you to use JSON-like syntax within Rust. The earlier example could be created in this way as follows:

```rs
let value = json!({
    "name": "John Doe",
    "age": 43,
    "phones": [
        "+44 1234567",
        "+44 2345678"
    ]
});
```

You can even include a number of types inside the macro and they will be converted to their JSON representations automatically, as follows:

```rs
let value = json!({
    "name": username,
    "age": (age_last_year + 1),
    "phones": [
        home_phone,
        work_phone
    ]
});
```

## Serializing Untyped JSON
To serialize a `Value` JSON type into its string representation, you can use either the `serialize` method or the `humphrey_json::to_string` method. The latter has the benefit that any type which can be converted to a value can be used, as you'll see in the next section.

```rs
let string = value.serialize();
```

You can also format the JSON with indentation and newlines using the `serialize_pretty` method, which takes the indent size as an argument.

```rs
let string = value.serialize_pretty(4);
```

## Conclusion
In this section we've looked at the tools available for working with untyped JSON values using Humphrey JSON. Next, we'll look at how to manipulate these values using Rust data structures.