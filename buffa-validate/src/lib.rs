mod violation;

pub mod helpers;

#[cfg(feature = "connectrpc")]
mod connect;

pub use violation::{Violation, Violations};

#[cfg(feature = "connectrpc")]
pub use connect::ValidateExt;

pub trait Validate {
    fn validate(&self) -> Result<(), Violations>;
}
