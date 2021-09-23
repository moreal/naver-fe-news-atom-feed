
use std::{io, time::SystemTimeError};

pub(crate) struct Error {
    reason: String,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("reason", &self.reason)
            .finish()
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error {
            reason: e.to_string(),
        }
    }
}

impl From<SystemTimeError> for Error {
    fn from(e: SystemTimeError) -> Self {
        Error {
            reason: e.to_string(),
        }
    }
}

impl From<atom_syndication::Error> for Error {
    fn from(e: atom_syndication::Error) -> Self {
        Error {
            reason: e.to_string(),
        }
    }
}
