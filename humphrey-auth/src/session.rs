//! Provides functionality for handling sessions and tokens.

use std::time::UNIX_EPOCH;

use rand_core::{OsRng, RngCore};

/// Represents a session, containing a token and an expiration time.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Session {
    /// The token string for this session.
    pub token: String,
    /// The UNIX timestamp at which this session will expire.
    pub expiry: u64,
}

impl Session {
    /// Creates a token with a lifetime of one hour.
    pub fn create() -> Self {
        Self::create_with_lifetime(3600)
    }

    /// Creates a token with the given lifetime (in seconds).
    pub fn create_with_lifetime(lifetime: u64) -> Self {
        let token = {
            let mut token: [u8; 32] = [0; 32];
            OsRng.fill_bytes(&mut token);
            token
        };

        let token_hex = token.iter().fold(String::with_capacity(64), |mut acc, &b| {
            acc.push_str(&format!("{:02x}", b));
            acc
        });

        let expiry = UNIX_EPOCH.elapsed().unwrap().as_secs() + lifetime;

        Self {
            token: token_hex,
            expiry,
        }
    }

    /// Returns true if the token is valid.
    pub fn valid(&self) -> bool {
        let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
        now < self.expiry
    }

    /// Returns true if the token has expired.
    pub fn expired(&self) -> bool {
        let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
        self.expiry < now
    }

    /// Refreshes the token, setting it to expire the given number of seconds after the current time.
    pub fn refresh(&mut self, lifetime: u64) {
        self.expiry = UNIX_EPOCH.elapsed().unwrap().as_secs() + lifetime;
    }
}
