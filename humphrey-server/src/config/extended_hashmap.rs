use crate::config::tree::ConfigNode;

use std::collections::HashMap;
use std::str::FromStr;

pub trait ExtendedMap<K, V> {
    /// Gets an item from the map by value.
    fn get_owned(&self, key: K) -> Option<V>;

    /// Gets an item from the map or returns a given default.
    fn get_optional(&self, key: K, default: V) -> V;

    /// Gets an item from the map or returns the given error.
    fn get_compulsory(&self, key: K, error: &'static str) -> Result<V, &'static str>;

    /// Gets an item from the map or a given default and parses it, or returns the given error.
    fn get_optional_parsed<T>(
        &self,
        key: K,
        default: T,
        error: &'static str,
    ) -> Result<T, &'static str>
    where
        T: FromStr;

    /// Gets an item from the map and parses it, or returns the given error.
    fn get_compulsory_parsed<T>(&self, key: K, error: &'static str) -> Result<T, &'static str>
    where
        T: FromStr;
}

impl ExtendedMap<&'static str, String> for HashMap<String, String> {
    fn get_owned(&self, key: &str) -> Option<String> {
        self.get(key).map(|s| s.to_string())
    }

    fn get_optional(&self, key: &str, default: String) -> String {
        self.get(key).unwrap_or(&default).to_string()
    }

    fn get_compulsory(&self, key: &str, error: &'static str) -> Result<String, &'static str> {
        self.get(key).map_or(Err(error), |s| Ok(s.clone()))
    }

    fn get_optional_parsed<T>(
        &self,
        key: &str,
        default: T,
        error: &'static str,
    ) -> Result<T, &'static str>
    where
        T: FromStr,
    {
        self.get(key)
            .map_or(Ok(default), |s| s.parse::<T>().map_err(|_| error))
    }

    fn get_compulsory_parsed<T>(&self, key: &str, error: &'static str) -> Result<T, &'static str>
    where
        T: FromStr,
    {
        self.get(key)
            .map_or(Err(error), |s| s.parse::<T>().map_err(|_| error))
    }
}

impl ExtendedMap<&'static str, String> for HashMap<String, ConfigNode> {
    fn get_owned(&self, key: &'static str) -> Option<String> {
        self.get(key).and_then(|n| n.get_string())
    }

    fn get_optional(&self, key: &'static str, default: String) -> String {
        self.get(key)
            .and_then(|n| n.get_string())
            .unwrap_or(default)
    }

    fn get_compulsory(
        &self,
        key: &'static str,
        error: &'static str,
    ) -> Result<String, &'static str> {
        self.get(key)
            .and_then(|n| n.get_string())
            .map_or(Err(error), Ok)
    }

    fn get_optional_parsed<T>(
        &self,
        key: &'static str,
        default: T,
        error: &'static str,
    ) -> Result<T, &'static str>
    where
        T: FromStr,
    {
        self.get(key)
            .map(|n| match n {
                ConfigNode::String(_, s) => s.parse().map_err(|_| ()),
                ConfigNode::Boolean(_, b) => b.parse().map_err(|_| ()),
                ConfigNode::Number(_, n) => n.parse().map_err(|_| ()),
                ConfigNode::Section(_, _) => Err(()),
                ConfigNode::Route(_, _) => Err(()),
                ConfigNode::Host(_, _) => Err(()),
            })
            .unwrap_or(Ok(default))
            .map_err(|_| error)
    }

    fn get_compulsory_parsed<T>(
        &self,
        key: &'static str,
        error: &'static str,
    ) -> Result<T, &'static str>
    where
        T: FromStr,
    {
        self.get(key)
            .map(|n| match n {
                ConfigNode::String(_, s) => s.parse().map_err(|_| ()),
                ConfigNode::Boolean(_, b) => b.parse().map_err(|_| ()),
                ConfigNode::Number(_, n) => n.parse().map_err(|_| ()),
                ConfigNode::Section(_, _) => Err(()),
                ConfigNode::Route(_, _) => Err(()),
                ConfigNode::Host(_, _) => Err(()),
            })
            .unwrap_or(Err(()))
            .map_err(|_| error)
    }
}
