use std::time::UNIX_EPOCH;

use rand_core::{OsRng, RngCore};

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Session {
    pub token: String,
    pub expiry: u64,
}

impl Session {
    pub fn create() -> Self {
        Self::create_with_lifetime(3600)
    }

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

    pub fn valid(&self) -> bool {
        let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
        now < self.expiry
    }

    pub fn expired(&self) -> bool {
        let now = UNIX_EPOCH.elapsed().unwrap().as_secs();
        self.expiry < now
    }

    pub fn refresh(&mut self, lifetime: u64) {
        self.expiry = UNIX_EPOCH.elapsed().unwrap().as_secs() + lifetime;
    }
}
