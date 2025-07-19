//! Error handling for the application

use std::fmt;

/// Main error type for the application
#[derive(Debug)]
pub enum AppError {
    /// I/O error
    Io(std::io::Error),
    /// Configuration error
    Config(String),
    /// System monitoring error
    System(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::System(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}