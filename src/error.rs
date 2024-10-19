use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct RouteAlreadyRegistered;

impl Error for RouteAlreadyRegistered {}

impl fmt::Display for RouteAlreadyRegistered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Route was already registered to other resource")
    }
}
