include!(concat!(env!("OUT_DIR"), "/_include.rs"));

#[cfg(test)]
mod tests {
    use super::test::v1::*;
    use buffa_validate::Validate;

    // ── User (basic string + numeric) ──────────────────────────────

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

    // ── Required ───────────────────────────────────────────────────

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

    // ── String constraints ─────────────────────────────────────────

    fn valid_string_constraints() -> StringConstraints {
        StringConstraints {
            exact_len: "abcde".into(),
            with_pattern: "hello".into(),
            with_prefix: "pre_value".into(),
            with_suffix: "value_suf".into(),
            with_contains: "has needle here".into(),
            not_contains: "clean text".into(),
            in_set: "a".into(),
            const_val: "fixed".into(),
            hostname: "example.com".into(),
            uuid: "550e8400-e29b-41d4-a716-446655440000".into(),
            ip_addr: "192.168.1.1".into(),
            uri: "https://example.com".into(),
            host_port: "example.com:443".into(),
            ..Default::default()
        }
    }

    #[test]
    fn string_constraints_valid_passes() {
        assert!(valid_string_constraints().validate().is_ok());
    }

    #[test]
    fn string_exact_len_fails() {
        let mut msg = valid_string_constraints();
        msg.exact_len = "abc".into();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "exact_len"));
    }

    #[test]
    fn string_pattern_fails() {
        let mut msg = valid_string_constraints();
        msg.with_pattern = "HELLO123".into();
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.field_path == "with_pattern")
        );
    }

    #[test]
    fn string_prefix_fails() {
        let mut msg = valid_string_constraints();
        msg.with_prefix = "nope".into();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "with_prefix"));
    }

    #[test]
    fn string_suffix_fails() {
        let mut msg = valid_string_constraints();
        msg.with_suffix = "nope".into();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "with_suffix"));
    }

    #[test]
    fn string_contains_fails() {
        let mut msg = valid_string_constraints();
        msg.with_contains = "no match".into();
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.field_path == "with_contains")
        );
    }

    #[test]
    fn string_not_contains_fails() {
        let mut msg = valid_string_constraints();
        msg.not_contains = "has bad word".into();
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.field_path == "not_contains")
        );
    }

    #[test]
    fn string_in_set_fails() {
        let mut msg = valid_string_constraints();
        msg.in_set = "z".into();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "in_set"));
    }

    #[test]
    fn string_const_fails() {
        let mut msg = valid_string_constraints();
        msg.const_val = "wrong".into();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "const_val"));
    }

    #[test]
    fn string_hostname_fails() {
        let mut msg = valid_string_constraints();
        msg.hostname = "not a host!".into();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "hostname"));
    }

    #[test]
    fn string_uuid_fails() {
        let mut msg = valid_string_constraints();
        msg.uuid = "not-a-uuid".into();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "uuid"));
    }

    #[test]
    fn string_ip_fails() {
        let mut msg = valid_string_constraints();
        msg.ip_addr = "999.999.999.999".into();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "ip_addr"));
    }

    #[test]
    fn string_uri_fails() {
        let mut msg = valid_string_constraints();
        msg.uri = "not a uri".into();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "uri"));
    }

    #[test]
    fn string_host_port_fails() {
        let mut msg = valid_string_constraints();
        msg.host_port = "justahostname".into();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "host_port"));
    }

    // ── Numeric constraints ────────────────────────────────────────

    fn valid_numeric_constraints() -> NumericConstraints {
        NumericConstraints {
            int_val: 50,
            long_val: 0,
            signed_val: 0,
            fixed_val: 100,
            sfixed_val: 5,
            dbl_val: 1.0,
            flt_val: 0.5,
            in_set: 3,
            not_in_set: 1,
            const_val: 42,
            ..Default::default()
        }
    }

    #[test]
    fn numeric_constraints_valid_passes() {
        assert!(valid_numeric_constraints().validate().is_ok());
    }

    #[test]
    fn int32_gt_fails() {
        let mut msg = valid_numeric_constraints();
        msg.int_val = 0;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "int_val"));
    }

    #[test]
    fn int32_lt_fails() {
        let mut msg = valid_numeric_constraints();
        msg.int_val = 100;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "int_val"));
    }

    #[test]
    fn int64_range_fails() {
        let mut msg = valid_numeric_constraints();
        msg.long_val = 5000;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "long_val"));
    }

    #[test]
    fn sint32_range_fails() {
        let mut msg = valid_numeric_constraints();
        msg.signed_val = -100;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "signed_val"));
    }

    #[test]
    fn fixed64_lte_fails() {
        let mut msg = valid_numeric_constraints();
        msg.fixed_val = 1000;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "fixed_val"));
    }

    #[test]
    fn sfixed32_range_fails() {
        let mut msg = valid_numeric_constraints();
        msg.sfixed_val = 11;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "sfixed_val"));
    }

    #[test]
    fn double_finite_fails() {
        let mut msg = valid_numeric_constraints();
        msg.dbl_val = f64::INFINITY;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "dbl_val"));
    }

    #[test]
    fn double_nan_fails() {
        let mut msg = valid_numeric_constraints();
        msg.dbl_val = f64::NAN;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "dbl_val"));
    }

    #[test]
    fn float_range_fails() {
        let mut msg = valid_numeric_constraints();
        msg.flt_val = 2.0;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "flt_val"));
    }

    #[test]
    fn uint32_in_fails() {
        let mut msg = valid_numeric_constraints();
        msg.in_set = 4;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "in_set"));
    }

    #[test]
    fn int32_not_in_fails() {
        let mut msg = valid_numeric_constraints();
        msg.not_in_set = 0;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "not_in_set"));
    }

    #[test]
    fn uint64_const_fails() {
        let mut msg = valid_numeric_constraints();
        msg.const_val = 99;
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "const_val"));
    }

    // ── Bool ───────────────────────────────────────────────────────

    #[test]
    fn bool_const_true_passes() {
        let msg = BoolConstraint {
            must_be_true: true,
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn bool_const_false_fails() {
        let msg = BoolConstraint::default();
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.field_path == "must_be_true")
        );
    }

    // ── Enum ───────────────────────────────────────────────────────

    #[test]
    fn enum_defined_value_passes() {
        let msg = EnumConstraint {
            status: Status::STATUS_ACTIVE.into(),
            must_active: Status::STATUS_ACTIVE.into(),
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn enum_in_fails() {
        let msg = EnumConstraint {
            status: Status::STATUS_ACTIVE.into(),
            must_active: Status::STATUS_INACTIVE.into(),
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "must_active"));
    }

    // ── Bytes ──────────────────────────────────────────────────────

    #[test]
    fn bytes_valid_passes() {
        let msg = BytesConstraint {
            data: vec![1, 2, 3],
            prefix_data: b"\x89PNG rest".to_vec(),
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn bytes_empty_fails() {
        let msg = BytesConstraint::default();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "data"));
    }

    #[test]
    fn bytes_too_long_fails() {
        let msg = BytesConstraint {
            data: vec![0u8; 300],
            prefix_data: b"\x89PNG".to_vec(),
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "data"));
    }

    #[test]
    fn bytes_wrong_prefix_fails() {
        let msg = BytesConstraint {
            data: vec![1],
            prefix_data: b"JPEG data".to_vec(),
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "prefix_data"));
    }

    // ── Ignore if default ──────────────────────────────────────────

    #[test]
    fn ignore_default_value_passes_when_default() {
        let msg = IgnoreDefaults::default();
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn ignore_default_value_validates_when_set() {
        let msg = IgnoreDefaults {
            name: "ab".into(),
            count: 5,
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "name"));
        assert!(err.violations.iter().any(|v| v.field_path == "count"));
    }

    #[test]
    fn ignore_default_value_passes_when_valid() {
        let msg = IgnoreDefaults {
            name: "alice".into(),
            count: 20,
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    // ── Repeated ───────────────────────────────────────────────────

    #[test]
    fn repeated_valid_passes() {
        let msg = RepeatedConstraint {
            tags: vec!["a".into()],
            unique_ids: vec![1, 2, 3],
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn repeated_too_few_fails() {
        let msg = RepeatedConstraint::default();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "tags"));
    }

    #[test]
    fn repeated_too_many_fails() {
        let msg = RepeatedConstraint {
            tags: vec![
                "a".into(),
                "b".into(),
                "c".into(),
                "d".into(),
                "e".into(),
                "f".into(),
            ],
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "tags"));
    }

    #[test]
    fn repeated_unique_fails() {
        let msg = RepeatedConstraint {
            tags: vec!["a".into()],
            unique_ids: vec![1, 2, 2],
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "unique_ids"));
    }

    // ── Map ────────────────────────────────────────────────────────

    #[test]
    fn map_valid_passes() {
        let mut msg = MapConstraint::default();
        msg.labels.insert("key".into(), "val".into());
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn map_empty_fails() {
        let msg = MapConstraint::default();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "labels"));
    }

    // ── Nested message validation ──────────────────────────────────

    #[test]
    fn nested_message_valid_passes() {
        let msg = Person {
            name: "Alice".into(),
            address: buffa::MessageField::some(Address {
                street: "123 Main St".into(),
                city: "Springfield".into(),
                zip: "12345".into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn nested_message_required_missing_fails() {
        let msg = Person {
            name: "Alice".into(),
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.field_path == "address" && v.rule == "required")
        );
    }

    #[test]
    fn nested_message_invalid_child_fails() {
        let msg = Person {
            name: "Alice".into(),
            address: buffa::MessageField::some(Address {
                street: "".into(),
                city: "Springfield".into(),
                zip: "bad".into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.field_path == "address.street")
        );
        assert!(err.violations.iter().any(|v| v.field_path == "address.zip"));
    }

    // ── Oneof required ─────────────────────────────────────────────

    #[test]
    fn oneof_required_set_passes() {
        let msg = OneofRequired {
            contact: Some(oneof_required::Contact::Email("a@b.com".into())),
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn oneof_required_unset_fails() {
        let msg = OneofRequired::default();
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "contact"));
    }

    // ── Map key/value validation ───────────────────────────────────

    #[test]
    fn map_key_value_valid_passes() {
        let mut msg = MapKeyValueConstraint::default();
        msg.metadata.insert("key".into(), "value".into());
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn map_empty_key_fails() {
        let mut msg = MapKeyValueConstraint::default();
        msg.metadata.insert("".into(), "value".into());
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.rule == "map.keys.string.min_len")
        );
    }

    #[test]
    fn map_empty_value_fails() {
        let mut msg = MapKeyValueConstraint::default();
        msg.metadata.insert("key".into(), "".into());
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.rule == "map.values.string.min_len")
        );
    }

    // ── CEL field-level constraints ────────────────────────────────

    #[test]
    fn cel_field_valid_passes() {
        let msg = CelFieldConstraint {
            code: "ABC".into(),
            score: 50,
            even_number: 4,
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn cel_field_code_format_fails() {
        let msg = CelFieldConstraint {
            code: "abc".into(),
            score: 50,
            even_number: 4,
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.field_path == "code" && v.rule == "code.format")
        );
    }

    #[test]
    fn cel_field_score_range_fails() {
        let msg = CelFieldConstraint {
            code: "ABC".into(),
            score: 101,
            even_number: 4,
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.field_path == "score" && v.rule == "score.range")
        );
    }

    #[test]
    fn cel_field_even_number_fails() {
        let msg = CelFieldConstraint {
            code: "ABC".into(),
            score: 50,
            even_number: 3,
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(
            err.violations
                .iter()
                .any(|v| v.field_path == "even_number" && v.rule == "even_number.even")
        );
    }

    // ── CEL message-level constraints ──────────────────────────────

    #[test]
    fn cel_message_valid_passes() {
        let msg = CelMessageConstraint {
            start_date: "2024-01-01".into(),
            end_date: "2024-12-31".into(),
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn cel_message_equal_dates_passes() {
        let msg = CelMessageConstraint {
            start_date: "2024-06-15".into(),
            end_date: "2024-06-15".into(),
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn cel_message_dates_order_fails() {
        let msg = CelMessageConstraint {
            start_date: "2024-12-31".into(),
            end_date: "2024-01-01".into(),
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.rule == "dates.order"));
    }

    // ── Connect-rust integration ───────────────────────────────────

    #[test]
    fn validate_ext_valid_passes() {
        use buffa_validate::ValidateExt;
        let msg = CreateUserRequest {
            name: "Alice".into(),
            email: "alice@example.com".into(),
            ..Default::default()
        };
        assert!(msg.validated().is_ok());
    }

    #[test]
    fn validate_ext_invalid_returns_connect_error() {
        use buffa_validate::ValidateExt;
        let msg = CreateUserRequest {
            name: "".into(),
            email: "bad".into(),
            ..Default::default()
        };
        let err = msg.validated().unwrap_err();
        assert_eq!(err.code, connectrpc::ErrorCode::InvalidArgument);
        assert!(!err.details.is_empty());
        let detail = &err.details[0];
        assert_eq!(
            detail.type_url,
            "type.googleapis.com/buf.validate.Violations"
        );
        let debug = detail.debug.as_ref().unwrap();
        let violations = debug["violations"].as_array().unwrap();
        assert!(violations.len() >= 2);
    }

    #[test]
    fn validated_service_wrapper_exists() {
        fn assert_impl<S: UserService>(_s: &ValidatedUserService<S>) {}
        struct DummyService;
        impl UserService for DummyService {
            async fn create_user<'a>(
                &'a self,
                _ctx: connectrpc::RequestContext,
                _request: buffa::view::OwnedView<
                    super::test::v1::__buffa::view::CreateUserRequestView<'static>,
                >,
            ) -> connectrpc::ServiceResult<
                impl connectrpc::Encodable<CreateUserResponse> + Send + use<'a>,
            > {
                connectrpc::Response::ok(CreateUserResponse {
                    id: "1".into(),
                    name: "test".into(),
                    ..Default::default()
                })
            }
            async fn get_user<'a>(
                &'a self,
                _ctx: connectrpc::RequestContext,
                _request: buffa::view::OwnedView<
                    super::test::v1::__buffa::view::GetUserRequestView<'static>,
                >,
            ) -> connectrpc::ServiceResult<
                impl connectrpc::Encodable<GetUserResponse> + Send + use<'a>,
            > {
                connectrpc::Response::ok(GetUserResponse {
                    id: "1".into(),
                    name: "test".into(),
                    ..Default::default()
                })
            }
        }
        let wrapped = ValidatedUserService(DummyService);
        assert_impl(&wrapped);
    }

    // ── View type validation ──────────────────────────────────────────

    #[test]
    fn view_user_valid_passes() {
        use super::test::v1::__buffa::view::UserView;
        let view = UserView {
            name: "Alice",
            email: "alice@example.com",
            age: 30,
            nickname: None,
            ..Default::default()
        };
        assert!(view.validate().is_ok());
    }

    #[test]
    fn view_user_empty_name_fails() {
        use super::test::v1::__buffa::view::UserView;
        let view = UserView {
            name: "",
            email: "alice@example.com",
            age: 30,
            ..Default::default()
        };
        let err = view.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "name"));
    }

    #[test]
    fn view_string_constraints_valid() {
        use super::test::v1::__buffa::view::StringConstraintsView;
        let view = StringConstraintsView {
            exact_len: "abcde",
            with_pattern: "abc",
            with_prefix: "pre_val",
            with_suffix: "val_suf",
            with_contains: "has needle here",
            not_contains: "good",
            in_set: "a",
            const_val: "fixed",
            hostname: "example.com",
            uuid: "550e8400-e29b-41d4-a716-446655440000",
            ip_addr: "192.168.1.1",
            uri: "https://example.com",
            host_port: "example.com:8080",
            ..Default::default()
        };
        assert!(view.validate().is_ok());
    }

    #[test]
    fn view_numeric_constraints_valid() {
        use super::test::v1::__buffa::view::NumericConstraintsView;
        let view = NumericConstraintsView {
            int_val: 50,
            long_val: 0,
            signed_val: 0,
            fixed_val: 100,
            sfixed_val: 5,
            dbl_val: 1.0,
            flt_val: 0.5,
            in_set: 3,
            not_in_set: 42,
            const_val: 42,
            ..Default::default()
        };
        assert!(view.validate().is_ok());
    }

    #[test]
    fn view_nested_person_valid() {
        use super::test::v1::__buffa::view::{AddressView, PersonView};
        use buffa::view::MessageFieldView;
        let view = PersonView {
            name: "Bob",
            address: MessageFieldView::set(AddressView {
                street: "123 Main St",
                city: "Springfield",
                zip: "12345",
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(view.validate().is_ok());
    }

    #[test]
    fn view_nested_person_invalid_zip() {
        use super::test::v1::__buffa::view::{AddressView, PersonView};
        use buffa::view::MessageFieldView;
        let view = PersonView {
            name: "Bob",
            address: MessageFieldView::set(AddressView {
                street: "123 Main St",
                city: "Springfield",
                zip: "bad",
                ..Default::default()
            }),
            ..Default::default()
        };
        let err = view.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "address.zip"));
    }

    #[test]
    fn view_owned_view_validates_directly() {
        use super::test::v1::__buffa::view::UserView;
        use buffa::Message;
        use buffa::view::OwnedView;

        let user = User {
            name: "Alice".into(),
            email: "alice@example.com".into(),
            age: 30,
            ..Default::default()
        };
        let bytes = buffa::bytes::Bytes::from(user.encode_to_vec());
        let owned_view = OwnedView::<UserView<'static>>::decode(bytes).unwrap();
        assert!(owned_view.validate().is_ok());
    }

    #[test]
    fn view_owned_view_invalid_fails() {
        use super::test::v1::__buffa::view::UserView;
        use buffa::Message;
        use buffa::view::OwnedView;

        let user = User {
            name: "".into(),
            email: "not-an-email".into(),
            age: 30,
            ..Default::default()
        };
        let bytes = buffa::bytes::Bytes::from(user.encode_to_vec());
        let owned_view = OwnedView::<UserView<'static>>::decode(bytes).unwrap();
        let err = owned_view.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "name"));
        assert!(err.violations.iter().any(|v| v.field_path == "email"));
    }

    // ── Duration constraints ──────────────────────────────────────────

    #[test]
    fn duration_valid_passes() {
        let msg = DurationConstraint {
            timeout: buffa::MessageField::some(buffa_types::google::protobuf::Duration {
                seconds: 30,
                nanos: 0,
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn duration_too_short_fails() {
        let msg = DurationConstraint {
            timeout: buffa::MessageField::some(buffa_types::google::protobuf::Duration {
                seconds: 0,
                nanos: 500_000_000,
                ..Default::default()
            }),
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "timeout"));
    }

    #[test]
    fn duration_too_long_fails() {
        let msg = DurationConstraint {
            timeout: buffa::MessageField::some(buffa_types::google::protobuf::Duration {
                seconds: 61,
                nanos: 0,
                ..Default::default()
            }),
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "timeout"));
    }

    #[test]
    fn duration_unset_skips_validation() {
        let msg = DurationConstraint::default();
        assert!(msg.validate().is_ok());
    }

    // ── Timestamp constraints ─────────────────────────────────────────

    #[test]
    fn timestamp_valid_passes() {
        let msg = TimestampConstraint {
            created_at: buffa::MessageField::some(buffa_types::google::protobuf::Timestamp {
                seconds: 1_700_000_000,
                nanos: 0,
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn timestamp_too_early_fails() {
        let msg = TimestampConstraint {
            created_at: buffa::MessageField::some(buffa_types::google::protobuf::Timestamp {
                seconds: 999_999_999,
                nanos: 0,
                ..Default::default()
            }),
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "created_at"));
    }

    #[test]
    fn timestamp_unset_skips_validation() {
        let msg = TimestampConstraint::default();
        assert!(msg.validate().is_ok());
    }

    // ── Any constraints ───────────────────────────────────────────────

    #[test]
    fn any_valid_type_url_passes() {
        let msg = AnyConstraint {
            payload: buffa::MessageField::some(buffa_types::google::protobuf::Any {
                type_url: "type.googleapis.com/test.v1.User".into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn any_invalid_type_url_fails() {
        let msg = AnyConstraint {
            payload: buffa::MessageField::some(buffa_types::google::protobuf::Any {
                type_url: "type.googleapis.com/test.v1.Unknown".into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let err = msg.validate().unwrap_err();
        assert!(err.violations.iter().any(|v| v.field_path == "payload"));
    }

    #[test]
    fn any_unset_skips_validation() {
        let msg = AnyConstraint::default();
        assert!(msg.validate().is_ok());
    }
}
