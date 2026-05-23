include!(concat!(env!("OUT_DIR"), "/_include.rs"));

#[cfg(test)]
mod tests {
    use super::test::v1::*;
    use buffa_validate::Validate;

    #[test]
    fn valid_user_passes() {
        let user = User {
            name: "Alice".into(),
            email: "alice@example.com".into(),
            age: 30,
            ..Default::default()
        };
        assert!(user.validate().is_ok());
    }

    #[test]
    fn empty_name_fails() {
        let user = User {
            name: "".into(),
            email: "alice@example.com".into(),
            age: 30,
            ..Default::default()
        };
        let err = user.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "name"));
    }

    #[test]
    fn invalid_email_fails() {
        let user = User {
            name: "Alice".into(),
            email: "not-an-email".into(),
            age: 30,
            ..Default::default()
        };
        let err = user.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "email"));
    }

    #[test]
    fn age_over_limit_fails() {
        let user = User {
            name: "Alice".into(),
            email: "alice@example.com".into(),
            age: 200,
            ..Default::default()
        };
        let err = user.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "age"));
    }

    #[test]
    fn optional_nickname_too_long_fails() {
        let user = User {
            name: "Alice".into(),
            email: "alice@example.com".into(),
            age: 30,
            nickname: Some("a]".repeat(20)),
            ..Default::default()
        };
        let err = user.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "nickname"));
    }

    #[test]
    fn required_field_missing_fails() {
        let msg = SimpleRequired::default();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "value"));
    }

    #[test]
    fn required_field_present_passes() {
        let msg = SimpleRequired {
            value: Some("hello".into()),
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }
}
