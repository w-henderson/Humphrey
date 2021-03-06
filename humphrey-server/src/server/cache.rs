//! Provides caching functionality.

use crate::config::Config;

use humphrey::http::mime::MimeType;
use std::{collections::VecDeque, time::SystemTime};

/// Represents the server's cache.
#[derive(Default)]
pub struct Cache {
    /// The cache's maximum size.
    pub cache_limit: usize,
    cache_time_limit: u64,
    cache_size: usize,
    data: VecDeque<CachedItem>,
}

/// Represents a cached item.
pub struct CachedItem {
    /// The route that this item was served at.
    pub route: String,
    /// The host that this item was served at.
    pub host: usize,
    /// The MIME type of the item.
    pub mime_type: MimeType,
    /// The time at which the item was cached.
    pub cache_time: u64,
    /// The item's data.
    pub data: Vec<u8>,
}

impl Cache {
    /// Attempts to get an item from the cache.
    /// If the item is not present, or it is stale, returns `None`.
    pub fn get(&self, route: &str, host: usize) -> Option<&CachedItem> {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let index = self
            .data
            .iter()
            .position(|item| item.route == route && item.host == host);

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
    pub fn set(&mut self, route: &str, host: usize, value: Vec<u8>, mime_type: MimeType) {
        while self.cache_size + value.len() > self.cache_limit {
            self.cache_size -= self.data[0].data.len();
            self.data.pop_front();
        }

        if let Some(existing_item) = self
            .data
            .iter()
            .position(|item| item.route == route && item.host == host)
        {
            self.cache_size -= self.data[existing_item].data.len();
            self.data.remove(existing_item);
        }

        self.cache_size += value.len();

        self.data.push_back(CachedItem {
            route: route.into(),
            host,
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
            cache_limit: config.cache.size_limit,
            cache_time_limit: config.cache.time_limit as u64,
            cache_size: 0,
            data: VecDeque::new(),
        }
    }
}
