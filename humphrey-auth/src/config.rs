/// Represents the configuration of the authentication provider.
#[derive(Clone)]
pub struct AuthConfig {
    pub(crate) default_lifetime: u64,
    pub(crate) default_refresh_lifetime: u64,
    pub(crate) pepper: Option<Vec<u8>>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            default_lifetime: 3600,
            default_refresh_lifetime: 3600,
            pepper: None,
        }
    }
}

impl AuthConfig {
    /// Sets the default lifetime for sessions.
    pub fn with_default_lifetime(mut self, lifetime: u64) -> Self {
        self.default_lifetime = lifetime;
        self
    }

    /// Sets the default lifetime for sessions after being refreshed.
    pub fn with_default_refresh_lifetime(mut self, lifetime: u64) -> Self {
        self.default_refresh_lifetime = lifetime;
        self
    }

    /// Sets the pepper used for hashing.
    pub fn with_pepper(mut self, pepper: impl AsRef<[u8]>) -> Self {
        self.pepper = Some(pepper.as_ref().to_vec());
        self
    }
}
