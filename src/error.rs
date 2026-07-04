use std::fmt;

/// Errors raised by elemtok.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// `length` (or `n`, for the raw sampling seam) was `0` or exceeded the
    /// upper bound.
    InvalidLength {
        /// The rejected value.
        got: usize,
        /// The largest accepted value (inclusive).
        max: usize,
    },
    /// The platform has no secure random source available.
    NoSecureRandom,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidLength { got, max } => write!(
                f,
                "elemtok: length must be an integer in [1, {max}], got {got}"
            ),
            Error::NoSecureRandom => {
                write!(f, "elemtok: no secure random source available")
            }
        }
    }
}

impl std::error::Error for Error {}
