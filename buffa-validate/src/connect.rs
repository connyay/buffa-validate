use crate::{Validate, Violations};

pub trait ValidateExt: Validate {
    fn validated(&self) -> Result<&Self, connectrpc::ConnectError> {
        self.validate().map_err(violations_to_connect_error)?;
        Ok(self)
    }
}

impl<T: Validate> ValidateExt for T {}

pub fn violations_to_connect_error(violations: Violations) -> connectrpc::ConnectError {
    let detail = connectrpc::error::ErrorDetail {
        type_url: "type.googleapis.com/buf.validate.Violations".to_string(),
        value: None,
        debug: Some(serde_json::json!({
            "violations": violations.violations.iter().map(|v| {
                serde_json::json!({
                    "fieldPath": v.field_path,
                    "constraintId": v.constraint_id,
                    "message": v.message,
                })
            }).collect::<Vec<_>>()
        })),
    };

    connectrpc::ConnectError::invalid_argument(format!("validation failed: {violations}"))
        .with_detail(detail)
}
