use std::fmt;

#[derive(Debug)]
pub enum OptionParsingError {
    InvalidKey(String),
    InvalidValue(String),
    NoValueFound,
}

impl fmt::Display for OptionParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey(key) => write!(f, "Invalid key encountered when parsing: {}", key),
            Self::InvalidValue(val) => write!(f, "Invalid value encountered when parsing: {}", val),
            Self::NoValueFound => write!(f, "Expected a value, found none"),
        }
    }
}
