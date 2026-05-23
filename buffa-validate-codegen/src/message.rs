use anyhow::Result;
use buffa_codegen::context::CodeGenContext;
use buffa_codegen::generated::descriptor::{DescriptorProto, FileDescriptorProto};
use proc_macro2::TokenStream;
use quote::quote;

use crate::constraints;
use crate::field;
use crate::rules;

pub fn generate_file_validations(
    file: &FileDescriptorProto,
    package: &str,
    ctx: &CodeGenContext<'_>,
) -> Result<TokenStream> {
    let mut tokens = TokenStream::new();

    for message in &file.message_type {
        let name = message.name.as_deref().unwrap_or("");
        let fqn = if package.is_empty() {
            format!(".{name}")
        } else {
            format!(".{package}.{name}")
        };
        generate_message_validation(message, &fqn, package, ctx, &mut tokens)?;
    }

    Ok(tokens)
}

fn generate_message_validation(
    message: &DescriptorProto,
    fqn: &str,
    _package: &str,
    ctx: &CodeGenContext<'_>,
    out: &mut TokenStream,
) -> Result<()> {
    let _message_name = message.name.as_deref().unwrap_or("");

    let rust_type_path = ctx
        .rust_type(fqn)
        .ok_or_else(|| anyhow::anyhow!("no rust type for {fqn}"))?;

    let type_tokens = buffa_codegen::idents::rust_path_to_tokens(rust_type_path);

    let mut field_checks = TokenStream::new();
    let mut has_any_rules = false;

    // Check message-level rules (CEL)
    if let Some(msg_rules) = rules::message_rules(message)
        && !msg_rules.cel.is_empty()
    {
        has_any_rules = true;
        let cel_checks = constraints::cel::generate_message_cel(&msg_rules.cel, &message.field)?;
        field_checks.extend(cel_checks);
    }

    // Check oneof-level rules
    for (idx, oneof_desc) in message.oneof_decl.iter().enumerate() {
        let has_real_fields = message
            .field
            .iter()
            .any(|f| f.oneof_index == Some(idx as i32) && !f.proto3_optional.unwrap_or(false));
        if !has_real_fields {
            continue;
        }

        if let Some(oneof_rules) = rules::oneof_rules(oneof_desc)
            && oneof_rules.required
        {
            has_any_rules = true;
            let oneof_name = oneof_desc.name.as_deref().unwrap_or("");
            let oneof_ident = buffa_codegen::idents::make_field_ident(oneof_name);
            field_checks.extend(constraints::oneof::generate_required(
                &oneof_ident,
                oneof_name,
            ));
        }
    }

    // Check field-level rules
    for field_desc in &message.field {
        let field_name = field_desc.name.as_deref().unwrap_or("");
        let is_message_type = field_desc.r#type
            == Some(
                buffa_codegen::generated::descriptor::field_descriptor_proto::Type::TYPE_MESSAGE,
            );
        let field_rules = rules::field_rules(field_desc);

        if let Some(ref fr) = field_rules {
            has_any_rules = true;
            let check = field::generate_field_validation(field_desc, fr, field_name, fqn, ctx)?;
            field_checks.extend(check);
        }

        if is_message_type {
            let field_type_name = field_desc.type_name.as_deref().unwrap_or("");
            let is_map_or_repeated = field_rules
                .as_ref()
                .and_then(|r| r.type_rules.as_ref())
                .is_some_and(|tr| {
                    matches!(
                        tr,
                        crate::generated::TypeRules::Map(_)
                            | crate::generated::TypeRules::Repeated(_)
                    )
                });
            let is_ignored = field_rules
                .as_ref()
                .is_some_and(|r| r.ignore == crate::generated::Ignore::Always);
            if !field_type_name.starts_with(".google.protobuf.")
                && !is_map_or_repeated
                && !is_ignored
            {
                has_any_rules = true;
                let field_ident = buffa_codegen::idents::make_field_ident(field_name);
                field_checks.extend(constraints::message::generate_nested_validation(
                    &field_ident,
                    field_name,
                    true,
                ));
            }
        }
    }

    // Validate repeated/map items
    for field_desc in &message.field {
        let field_name = field_desc.name.as_deref().unwrap_or("");
        let is_message_type = field_desc.r#type
            == Some(
                buffa_codegen::generated::descriptor::field_descriptor_proto::Type::TYPE_MESSAGE,
            );

        if let Some(field_rules) = rules::field_rules(field_desc) {
            if field_rules.ignore == crate::generated::Ignore::Always {
                continue;
            }

            if let Some(crate::generated::TypeRules::Repeated(ref repeated_rules)) =
                field_rules.type_rules
            {
                if is_message_type {
                    let field_type_name = field_desc.type_name.as_deref().unwrap_or("");
                    if !field_type_name.starts_with(".google.protobuf.") {
                        has_any_rules = true;
                        let field_ident = buffa_codegen::idents::make_field_ident(field_name);
                        field_checks.extend(quote! {
                            for (__idx, __item) in self.#field_ident.iter().enumerate() {
                                if let ::core::result::Result::Err(nested_violations) = ::buffa_validate::Validate::validate(__item) {
                                    for mut v in nested_violations.violations {
                                        if v.field_path.is_empty() {
                                            v.field_path = ::std::format!("{}[{}]", #field_name, __idx);
                                        } else {
                                            v.field_path = ::std::format!("{}[{}].{}", #field_name, __idx, v.field_path);
                                        }
                                        violations.push(v);
                                    }
                                }
                            }
                        });
                    }
                }

                if let Some(ref items) = repeated_rules.items
                    && let Some(ref type_rules) = items.type_rules
                    && !is_message_type
                {
                    let item_checks = generate_repeated_item_type_checks(type_rules, field_name)?;
                    if !item_checks.is_empty() {
                        has_any_rules = true;
                        let field_ident = buffa_codegen::idents::make_field_ident(field_name);
                        field_checks.extend(quote! {
                            for (__idx, __item) in self.#field_ident.iter().enumerate() {
                                let __item_path = ::std::format!("{}[{}]", #field_name, __idx);
                                #item_checks
                            }
                        });
                    }
                }
            }
            if let Some(crate::generated::TypeRules::Map(ref map_rules)) = field_rules.type_rules
                && (map_rules.keys.is_some() || map_rules.values.is_some())
            {
                let field_ident = buffa_codegen::idents::make_field_ident(field_name);
                let mut entry_checks = TokenStream::new();

                if let Some(ref keys_rules) = map_rules.keys
                    && let Some(ref type_rules) = keys_rules.type_rules
                {
                    let key_checks = generate_map_key_checks(type_rules, field_name)?;
                    entry_checks.extend(key_checks);
                }
                if let Some(ref values_rules) = map_rules.values
                    && let Some(ref type_rules) = values_rules.type_rules
                {
                    let val_checks = generate_map_value_checks(type_rules, field_name)?;
                    entry_checks.extend(val_checks);
                }

                if !entry_checks.is_empty() {
                    has_any_rules = true;
                    field_checks.extend(quote! {
                        for (__key, __val) in &self.#field_ident {
                            #entry_checks
                        }
                    });
                }
            }
        }
    }

    if has_any_rules {
        out.extend(quote! {
            impl ::buffa_validate::Validate for #type_tokens {
                fn validate(&self) -> ::core::result::Result<(), ::buffa_validate::Violations> {
                    let mut violations = ::std::vec::Vec::new();
                    #field_checks
                    if violations.is_empty() {
                        ::core::result::Result::Ok(())
                    } else {
                        ::core::result::Result::Err(::buffa_validate::Violations { violations })
                    }
                }
            }
        });

        if ctx.config.generate_views
            && let Some(view_tokens) = view_type_tokens(rust_type_path)
        {
            let field_checks_view = field_checks.clone();
            out.extend(quote! {
                impl ::buffa_validate::Validate for #view_tokens<'_> {
                    fn validate(&self) -> ::core::result::Result<(), ::buffa_validate::Violations> {
                        let mut violations = ::std::vec::Vec::new();
                        #field_checks_view
                        if violations.is_empty() {
                            ::core::result::Result::Ok(())
                        } else {
                            ::core::result::Result::Err(::buffa_validate::Violations { violations })
                        }
                    }
                }
            });
        }
    }

    // Recurse into nested messages
    for nested in &message.nested_type {
        let nested_name = nested.name.as_deref().unwrap_or("");
        let nested_fqn = format!("{fqn}.{nested_name}");
        generate_message_validation(nested, &nested_fqn, _package, ctx, out)?;
    }

    Ok(())
}

use crate::generated::TypeRules;

fn view_type_tokens(owned_rust_path: &str) -> Option<TokenStream> {
    let pos = owned_rust_path.rfind("::")?;
    let prefix = &owned_rust_path[..pos];
    let name = &owned_rust_path[pos + 2..];
    let view_path = format!(
        "{}::{}::view::{}View",
        prefix,
        buffa_codegen::context::SENTINEL_MOD,
        name
    );
    Some(buffa_codegen::idents::rust_path_to_tokens(&view_path))
}

fn generate_map_key_checks(type_rules: &TypeRules, field_name: &str) -> Result<TokenStream> {
    match type_rules {
        TypeRules::String(rules) => {
            let mut checks = TokenStream::new();
            if let Some(min) = rules.min_len {
                checks.extend(quote! {
                    if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_min_len(__key, #min) {
                        let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                        violations.push(::buffa_validate::Violation::new(__path, "map.keys.string.min_len", msg));
                    }
                });
            }
            if let Some(max) = rules.max_len {
                checks.extend(quote! {
                    if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_max_len(__key, #max) {
                        let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                        violations.push(::buffa_validate::Violation::new(__path, "map.keys.string.max_len", msg));
                    }
                });
            }
            if let Some(ref pattern) = rules.pattern {
                checks.extend(quote! {
                    {
                        static __RE: ::std::sync::OnceLock<::buffa_validate::__private::Regex> = ::std::sync::OnceLock::new();
                        if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_pattern(__key, &__RE, #pattern) {
                            let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                            violations.push(::buffa_validate::Violation::new(__path, "map.keys.string.pattern", msg));
                        }
                    }
                });
            }
            Ok(checks)
        }
        TypeRules::Int32(rules) => generate_map_numeric_inline(rules, field_name, true, "map.keys"),
        TypeRules::Int64(rules) => generate_map_numeric_inline(rules, field_name, true, "map.keys"),
        TypeRules::Uint32(rules) => {
            generate_map_numeric_inline(rules, field_name, true, "map.keys")
        }
        TypeRules::Uint64(rules) => {
            generate_map_numeric_inline(rules, field_name, true, "map.keys")
        }
        TypeRules::Sint32(rules) => {
            generate_map_numeric_inline(rules, field_name, true, "map.keys")
        }
        TypeRules::Sint64(rules) => {
            generate_map_numeric_inline(rules, field_name, true, "map.keys")
        }
        TypeRules::Fixed32(rules) => {
            generate_map_numeric_inline(rules, field_name, true, "map.keys")
        }
        TypeRules::Fixed64(rules) => {
            generate_map_numeric_inline(rules, field_name, true, "map.keys")
        }
        TypeRules::Sfixed32(rules) => {
            generate_map_numeric_inline(rules, field_name, true, "map.keys")
        }
        TypeRules::Sfixed64(rules) => {
            generate_map_numeric_inline(rules, field_name, true, "map.keys")
        }
        TypeRules::Bool(rules) => generate_map_bool_inline(rules, field_name, true, "map.keys"),
        _ => Ok(TokenStream::new()),
    }
}

fn generate_map_value_checks(type_rules: &TypeRules, field_name: &str) -> Result<TokenStream> {
    match type_rules {
        TypeRules::String(rules) => {
            let mut checks = TokenStream::new();
            if let Some(min) = rules.min_len {
                checks.extend(quote! {
                    if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_min_len(__val, #min) {
                        let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                        violations.push(::buffa_validate::Violation::new(__path, "map.values.string.min_len", msg));
                    }
                });
            }
            if let Some(max) = rules.max_len {
                checks.extend(quote! {
                    if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_max_len(__val, #max) {
                        let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                        violations.push(::buffa_validate::Violation::new(__path, "map.values.string.max_len", msg));
                    }
                });
            }
            Ok(checks)
        }
        TypeRules::Int32(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Int64(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Uint32(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Uint64(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Sint32(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Sint64(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Fixed32(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Fixed64(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Sfixed32(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Sfixed64(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Float(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Double(rules) => {
            generate_map_numeric_inline(rules, field_name, false, "map.values")
        }
        TypeRules::Bool(rules) => generate_map_bool_inline(rules, field_name, false, "map.values"),
        TypeRules::Enum(rules) => generate_map_enum_inline(rules, field_name, "map.values"),
        _ => Ok(TokenStream::new()),
    }
}

fn generate_map_numeric_inline<T: quote::ToTokens + std::fmt::Display>(
    rules: &crate::generated::NumericRules<T>,
    field_name: &str,
    is_key: bool,
    prefix: &str,
) -> Result<TokenStream> {
    let mut checks = TokenStream::new();
    let val_expr = if is_key {
        quote! { *__key }
    } else {
        quote! { *__val }
    };

    if let Some(ref c) = rules.r#const {
        let rule_id = format!("{prefix}.const");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_const(#val_expr, #c) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    if let Some(ref bound) = rules.gt {
        let rule_id = format!("{prefix}.gt");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_gt(#val_expr, #bound) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    if let Some(ref bound) = rules.gte {
        let rule_id = format!("{prefix}.gte");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_gte(#val_expr, #bound) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    if let Some(ref bound) = rules.lt {
        let rule_id = format!("{prefix}.lt");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_lt(#val_expr, #bound) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    if let Some(ref bound) = rules.lte {
        let rule_id = format!("{prefix}.lte");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_lte(#val_expr, #bound) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    if !rules.r#in.is_empty() {
        let values = &rules.r#in;
        let rule_id = format!("{prefix}.in");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_in(&#val_expr, &[#(#values),*]) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    if !rules.not_in.is_empty() {
        let values = &rules.not_in;
        let rule_id = format!("{prefix}.not_in");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_not_in(&#val_expr, &[#(#values),*]) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    Ok(checks)
}

fn generate_map_bool_inline(
    rules: &crate::generated::BoolRules,
    field_name: &str,
    is_key: bool,
    prefix: &str,
) -> Result<TokenStream> {
    let mut checks = TokenStream::new();
    let val_expr = if is_key {
        quote! { *__key }
    } else {
        quote! { *__val }
    };
    if let Some(expected) = rules.r#const {
        let rule_id = format!("{prefix}.bool.const");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_bool_const(#val_expr, #expected) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    Ok(checks)
}

fn generate_map_enum_inline(
    rules: &crate::generated::EnumRules,
    field_name: &str,
    prefix: &str,
) -> Result<TokenStream> {
    let mut checks = TokenStream::new();
    if let Some(c) = rules.r#const {
        let rule_id = format!("{prefix}.enum.const");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_const(__val.to_i32(), #c) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    if rules.defined_only {
        let rule_id = format!("{prefix}.enum.defined_only");
        checks.extend(quote! {
            if __val.is_unknown() {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, "value must be a defined enum value"));
            }
        });
    }
    if !rules.r#in.is_empty() {
        let values = &rules.r#in;
        let rule_id = format!("{prefix}.enum.in");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_in(&__val.to_i32(), &[#(#values),*]) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    if !rules.not_in.is_empty() {
        let values = &rules.not_in;
        let rule_id = format!("{prefix}.enum.not_in");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_not_in(&__val.to_i32(), &[#(#values),*]) {
                let __path = ::std::format!("{}[{:?}]", #field_name, __key);
                violations.push(::buffa_validate::Violation::new(__path, #rule_id, msg));
            }
        });
    }
    Ok(checks)
}

fn generate_repeated_item_type_checks(
    type_rules: &TypeRules,
    _field_name: &str,
) -> Result<TokenStream> {
    match type_rules {
        TypeRules::String(rules) => generate_repeated_string_checks(rules),
        TypeRules::Int32(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Int64(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Uint32(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Uint64(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Sint32(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Sint64(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Fixed32(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Fixed64(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Sfixed32(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Sfixed64(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Float(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Double(rules) => generate_repeated_numeric_checks(rules, "repeated.items"),
        TypeRules::Bool(rules) => {
            let mut checks = TokenStream::new();
            if let Some(expected) = rules.r#const {
                checks.extend(quote! {
                    if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_bool_const(*__item, #expected) {
                        violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.bool.const", msg));
                    }
                });
            }
            Ok(checks)
        }
        TypeRules::Enum(rules) => {
            let mut checks = TokenStream::new();
            if rules.defined_only {
                checks.extend(quote! {
                    if __item.is_unknown() {
                        violations.push(::buffa_validate::Violation::new(
                            __item_path.clone(), "repeated.items.enum.defined_only", "value must be a defined enum value",
                        ));
                    }
                });
            }
            if let Some(c) = rules.r#const {
                checks.extend(quote! {
                    if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_const(__item.to_i32(), #c) {
                        violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.enum.const", msg));
                    }
                });
            }
            if !rules.r#in.is_empty() {
                let values = &rules.r#in;
                checks.extend(quote! {
                    if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_in(&__item.to_i32(), &[#(#values),*]) {
                        violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.enum.in", msg));
                    }
                });
            }
            if !rules.not_in.is_empty() {
                let values = &rules.not_in;
                checks.extend(quote! {
                    if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_not_in(&__item.to_i32(), &[#(#values),*]) {
                        violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.enum.not_in", msg));
                    }
                });
            }
            Ok(checks)
        }
        TypeRules::Bytes(rules) => generate_repeated_bytes_checks(rules),
        _ => Ok(TokenStream::new()),
    }
}

fn generate_repeated_string_checks(rules: &crate::generated::StringRules) -> Result<TokenStream> {
    let mut checks = TokenStream::new();
    if let Some(ref c) = rules.r#const {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_const(__item, #c) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.const", msg));
            }
        });
    }
    if let Some(len) = rules.len {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_len(__item, #len) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.len", msg));
            }
        });
    }
    if let Some(min) = rules.min_len {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_min_len(__item, #min) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.min_len", msg));
            }
        });
    }
    if let Some(max) = rules.max_len {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_max_len(__item, #max) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.max_len", msg));
            }
        });
    }
    if let Some(ref pattern) = rules.pattern {
        checks.extend(quote! {
            {
                static __RE: ::std::sync::OnceLock<::buffa_validate::__private::Regex> = ::std::sync::OnceLock::new();
                if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_pattern(__item, &__RE, #pattern) {
                    violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.pattern", msg));
                }
            }
        });
    }
    if let Some(ref prefix) = rules.prefix {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_prefix(__item, #prefix) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.prefix", msg));
            }
        });
    }
    if let Some(ref suffix) = rules.suffix {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_suffix(__item, #suffix) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.suffix", msg));
            }
        });
    }
    if let Some(ref contains) = rules.contains {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_contains(__item, #contains) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.contains", msg));
            }
        });
    }
    if let Some(ref not_contains) = rules.not_contains {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_not_contains(__item, #not_contains) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.not_contains", msg));
            }
        });
    }
    if !rules.r#in.is_empty() {
        let values: Vec<&str> = rules.r#in.iter().map(|s| s.as_str()).collect();
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_in(__item, &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.in", msg));
            }
        });
    }
    if !rules.not_in.is_empty() {
        let values: Vec<&str> = rules.not_in.iter().map(|s| s.as_str()).collect();
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_not_in(__item, &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), "repeated.items.string.not_in", msg));
            }
        });
    }
    Ok(checks)
}

fn generate_repeated_numeric_checks<T: quote::ToTokens + std::fmt::Display>(
    rules: &crate::generated::NumericRules<T>,
    prefix: &str,
) -> Result<TokenStream> {
    let mut checks = TokenStream::new();
    if let Some(ref c) = rules.r#const {
        let rule_id = format!("{prefix}.const");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_const(*__item, #c) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), #rule_id, msg));
            }
        });
    }
    if let Some(ref bound) = rules.gt {
        let rule_id = format!("{prefix}.gt");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_gt(*__item, #bound) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), #rule_id, msg));
            }
        });
    }
    if let Some(ref bound) = rules.gte {
        let rule_id = format!("{prefix}.gte");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_gte(*__item, #bound) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), #rule_id, msg));
            }
        });
    }
    if let Some(ref bound) = rules.lt {
        let rule_id = format!("{prefix}.lt");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_lt(*__item, #bound) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), #rule_id, msg));
            }
        });
    }
    if let Some(ref bound) = rules.lte {
        let rule_id = format!("{prefix}.lte");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_lte(*__item, #bound) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), #rule_id, msg));
            }
        });
    }
    if !rules.r#in.is_empty() {
        let values = &rules.r#in;
        let rule_id = format!("{prefix}.in");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_in(&*__item, &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), #rule_id, msg));
            }
        });
    }
    if !rules.not_in.is_empty() {
        let values = &rules.not_in;
        let rule_id = format!("{prefix}.not_in");
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_not_in(&*__item, &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(__item_path.clone(), #rule_id, msg));
            }
        });
    }
    Ok(checks)
}

fn generate_repeated_bytes_checks(rules: &crate::generated::BytesRules) -> Result<TokenStream> {
    let mut checks = TokenStream::new();
    if let Some(len) = rules.len {
        let len_usize = len as usize;
        checks.extend(quote! {
            if __item.len() != #len_usize {
                violations.push(::buffa_validate::Violation::new(
                    __item_path.clone(), "repeated.items.bytes.len",
                    ::std::format!("value byte length must be exactly {}", #len),
                ));
            }
        });
    }
    if let Some(min) = rules.min_len {
        let min_usize = min as usize;
        checks.extend(quote! {
            if __item.len() < #min_usize {
                violations.push(::buffa_validate::Violation::new(
                    __item_path.clone(), "repeated.items.bytes.min_len",
                    ::std::format!("value byte length must be at least {}", #min),
                ));
            }
        });
    }
    if let Some(max) = rules.max_len {
        let max_usize = max as usize;
        checks.extend(quote! {
            if __item.len() > #max_usize {
                violations.push(::buffa_validate::Violation::new(
                    __item_path.clone(), "repeated.items.bytes.max_len",
                    ::std::format!("value byte length must be at most {}", #max),
                ));
            }
        });
    }
    Ok(checks)
}
