use crate::config::Config;

use humphrey::http::mime::MimeType;
use std::{collections::VecDeque, time::SystemTime};

/// Represents the server's cache.
#[derive(Default)]
pub struct Cache {
    pub cache_limit: usize,
    cache_time_limit: u64,
    cache_size: usize,
    data: VecDeque<CachedItem>,
}

/// Represents a cached item.
pub struct CachedItem {
    pub route: String,
    pub mime_type: MimeType,
    pub cache_time: u64,
    pub data: Vec<u8>,
}

impl Cache {
    /// Attempts to get an item from the cache.
    /// If the item is not present, or it is stale, returns `None`.
    pub fn get(&self, route: &str) -> Option<&CachedItem> {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let index = self.data.iter().position(|item| item.route == route);

        if let Some(index) = index {
            let item = &self.data[index];
            if time - item.cache_time > self.cache_time_limit {
                None
            } else {
                Some(item)
            }
        } else {
            None
        }
    }

    /// Sets an item in the cache.
    /// Overwrites older versions if needed.
    pub fn set(&mut self, route: &str, value: Vec<u8>, mime_type: MimeType) {
        while self.cache_size + value.len() > self.cache_limit {
            self.cache_size -= self.data[0].data.len();
            self.data.pop_front();
        }

        if let Some(existing_item) = self.data.iter().position(|item| item.route == route) {
            self.cache_size -= self.data[existing_item].data.len();
            self.data.remove(existing_item);
        }

        self.cache_size += value.len();

        self.data.push_back(CachedItem {
            route: route.into(),
            data: value,
            mime_type,
            cache_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
    }
}

impl From<&Config> for Cache {
    fn from(config: &Config) -> Self {
        Self {
            cache_limit: config.cache_limit,
            cache_time_limit: config.cache_time_limit,
            cache_size: 0,
            data: VecDeque::new(),
        }
    }
}
