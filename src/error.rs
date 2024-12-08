use std::fmt;

#[derive(Debug)]
enum OptionParsingError {
    InvalidKey(String),
    InvalidValue(String),
}

impl fmt::Display for OptionParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey(key) => write!(f, "Invalid key encountered when parsing: {}", key),
            Self::InvalidValue(val) => write!(f, "Invalid value encountered when parsing: {}", val),
        }
    }
}
