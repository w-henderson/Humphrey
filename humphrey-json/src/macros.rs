//! Provides the `json!` macro for creating JSON values.

/// Create a JSON value from JSON-like syntax.
///
/// ## Usage
/// The macro allows you to create any kind of JSON value.
///
/// ```rs
/// let value = json!({
///     "key": "value",
///     "array": [1, 2, 3],
///     "empty": null,
///     "bool": true
/// });
/// ```
///
/// It can also be used as a shorthand to create literal values.
///
/// ```rs
/// assert_eq!(json!(true), Value::Bool(true));
/// assert_eq!(json!(1234), Value::Number(1234.0));
/// assert_eq!(json!("Hello, world!"), Value::String("Hello, world!".into()));
/// ```
#[macro_export]
macro_rules! json {
    () => {
        $crate::Value::Null
    };
    (null) => {
        $crate::Value::Null
    };
    ([ $( $el:tt ),* ]) => {
        $crate::Value::Array(vec![ $( json!($el) ),* ])
    };
    ({}) => {
        $crate::Value::Object(Vec::new())
    };
    ({ $( $k:tt : $v:tt ),* }) => {
        $crate::Value::Object(vec![
            $( ($k.to_string(), json!($v)) ),*
        ])
    };
    ($v:tt) => {
        $crate::Value::from($v)
    };
}
