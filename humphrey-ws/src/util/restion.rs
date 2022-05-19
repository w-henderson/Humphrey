//! Provides the `Restion` type, a combination of `Result` and `Option`.

use std::fmt::Debug;

/// A combination of `Result` and `Option`, used to represent the result of non-blocking operations.
#[derive(Debug)]
pub enum Restion<T, E> {
    /// A successful result, equivalent to `Option::Some` and `Result::Ok`.
    Ok(T),
    /// An unsuccessful result, equivalent to `Result::Err`.
    Err(E),
    /// No value was available, equivalent to `Option::None`.
    None,
}

impl<T, E> Restion<T, E> {
    /// Returns `true` if the result is `Ok`.
    pub const fn is_ok(&self) -> bool {
        matches!(self, Restion::Ok(_))
    }

    /// Returns `true` if the result is `Err`.
    pub const fn is_err(&self) -> bool {
        matches!(self, Restion::Err(_))
    }

    /// Returns `true` if the result is `None`.
    pub const fn is_none(&self) -> bool {
        matches!(self, Restion::None)
    }
}

impl<T, E: Debug> Restion<T, E> {
    /// Returns the contained `Ok` value, consuming the `self` value.
    ///
    /// ## Panics
    /// Panics if the self value equals `Err` or `None`.
    pub fn unwrap(self) -> T {
        match self {
            Restion::Ok(t) => t,
            Restion::Err(e) => panic!("{:?}", e),
            Restion::None => panic!("called `Restion::unwrap()` on a `None` value"),
        }
    }
}

impl<T, E> From<Result<T, E>> for Restion<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(t) => Restion::Ok(t),
            Err(e) => Restion::Err(e),
        }
    }
}

impl<T> From<Option<T>> for Restion<T, ()> {
    fn from(option: Option<T>) -> Self {
        match option {
            Some(t) => Restion::Ok(t),
            None => Restion::None,
        }
    }
}
