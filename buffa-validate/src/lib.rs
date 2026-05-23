mod violation;

pub mod helpers;

#[cfg(feature = "connectrpc")]
mod connect;

pub use violation::{Violation, Violations};

#[cfg(feature = "connectrpc")]
pub use connect::{ValidateExt, violations_to_connect_error};

pub trait Validate {
    fn validate(&self) -> Result<(), Violations>;
}

#[doc(hidden)]
pub mod __private {
    pub use cel;
    pub use regex::Regex;
}
