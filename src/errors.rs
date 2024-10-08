use std::{error::Error, fmt};

#[derive(Debug)]
pub enum PaymentError {
    InvalidCliArgument(String),
    CsvParseError(String),
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
