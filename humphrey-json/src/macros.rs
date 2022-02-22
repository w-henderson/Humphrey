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
        $crate::Value::Object(::std::collections::HashMap::new())
    };
    ({ $( $k:tt : $v:tt ),* }) => {
        {
            let mut map = ::std::collections::HashMap::new();
            $(
                map.insert($k.to_string(), json!($v));
            )*
            $crate::Value::Object(map)
        }
    };
    ($v:tt) => {
        $crate::Value::from($v)
    };
}
