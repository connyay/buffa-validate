use crate::{Validate, Violations};

pub trait ValidateExt: Validate {
    fn validated(&self) -> Result<&Self, connectrpc::ConnectError> {
        self.validate().map_err(violations_to_connect_error)?;
        Ok(self)
    }
}

impl<T: Validate> ValidateExt for T {}

fn violations_to_connect_error(violations: Violations) -> connectrpc::ConnectError {
    connectrpc::ConnectError::invalid_argument(format!("validation failed: {violations}"))
}
