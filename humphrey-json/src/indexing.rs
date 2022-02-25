use crate::Value;

use std::ops;

pub trait Index {
    fn json_index<'v>(&self, v: &'v Value) -> Option<&'v Value>;
    fn json_index_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value>;
}

impl Index for usize {
    fn json_index<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        match v {
            Value::Array(a) => a.get(*self),
            _ => None,
        }
    }

    fn json_index_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        match v {
            Value::Array(a) => a.get_mut(*self),
            _ => None,
        }
    }
}

impl Index for &str {
    fn json_index<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        match v {
            Value::Object(o) => o.iter().find(|(k, _)| k == self).map(|(_, v)| v),
            _ => None,
        }
    }

    fn json_index_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        match v {
            Value::Object(o) => {
                if let Some(i) = o.iter().position(|(k, _)| k == self) {
                    Some(&mut o[i].1)
                } else {
                    o.push((self.to_string(), Value::Null));
                    o.last_mut().map(|(_, v)| v)
                }
            }
            _ => None,
        }
    }
}

impl Index for String {
    fn json_index<'v>(&self, v: &'v Value) -> Option<&'v Value> {
        self.as_str().json_index(v)
    }

    fn json_index_mut<'v>(&self, v: &'v mut Value) -> Option<&'v mut Value> {
        self.as_str().json_index_mut(v)
    }
}

impl<T> ops::Index<T> for Value
where
    T: Index,
{
    type Output = Value;

    fn index(&self, index: T) -> &Self::Output {
        index.json_index(self).unwrap_or(&Value::Null)
    }
}

impl<T> ops::IndexMut<T> for Value
where
    T: Index,
{
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        index
            .json_index_mut(self)
            .expect("Cannot get mutable index")
    }
}
