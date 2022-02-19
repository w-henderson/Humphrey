use crate::Value;

use std::collections::HashMap;

pub fn to_object(vec: Vec<(&'static str, Value)>) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    for (key, value) in vec {
        map.insert(key.to_string(), value);
    }

    map
}
