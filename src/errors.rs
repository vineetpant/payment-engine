use std::{error::Error, fmt};

/// Represents the various errors that can occur in the payment engine.
#[derive(Debug)]
pub enum PaymentError {
    /// Indicates invalid cli argument.
    InvalidCliArgument(String),
    /// Indicates error in csv parsing.
    CsvParseError(String),
    /// Indicates error in opening or reading the csv file.
    FileError(String),
}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PaymentError::InvalidCliArgument(msg) => write!(f, "Invalid cli argument: {}", msg),
            PaymentError::CsvParseError(msg) => write!(f, "CSV parse error: {}", msg),
            PaymentError::FileError(msg) => write!(f, "File error: {}", msg),
        }
    }
}

impl Error for PaymentError {}
