mod base64;

use std::fmt;

pub use crate::base64::Base64;

pub trait Transform {
    fn name(&self) -> &'static str;

    fn encode(&self, input: &[u8]) -> String;

    /// `None` when the transform is one-way.
    fn decode(&self, _input: &str) -> Option<Result<Vec<u8>, DecodeError>> {
        None
    }
}

/// Displayed to the user as is, so messages are short and lower-case.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecodeError {
    InvalidChar(char),
    InvalidLastChar(char),
    MixedAlphabets,
    InvalidPadding,
    Incomplete,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChar(c) => write!(f, "invalid character {c:?}"),
            Self::InvalidLastChar(c) => write!(f, "invalid last character {c:?}"),
            Self::MixedAlphabets => f.write_str("mixes standard and URL-safe characters"),
            Self::InvalidPadding => f.write_str("misplaced padding"),
            Self::Incomplete => f.write_str("incomplete input"),
        }
    }
}

impl std::error::Error for DecodeError {}
