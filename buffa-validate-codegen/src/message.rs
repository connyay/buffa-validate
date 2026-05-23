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
    package: &str,
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
    if let Some(_msg_rules) = rules::message_rules(message) {
        // CEL support will be added in Phase 4
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
            && oneof_rules.required {
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

        if let Some(field_rules) = rules::field_rules(field_desc) {
            has_any_rules = true;
            let check =
                field::generate_field_validation(field_desc, &field_rules, field_name, fqn, ctx)?;
            field_checks.extend(check);
        }

        if is_message_type {
            let field_type_name = field_desc.type_name.as_deref().unwrap_or("");
            let is_map_or_repeated = rules::field_rules(field_desc)
                .as_ref()
                .and_then(|r| r.type_rules.as_ref())
                .is_some_and(|tr| {
                    matches!(
                        tr,
                        crate::generated::TypeRules::Map(_)
                            | crate::generated::TypeRules::Repeated(_)
                    )
                });
            if !field_type_name.starts_with(".google.protobuf.")
                && !is_map_or_repeated
                && rules::field_rules(field_desc).is_some()
            {
                let field_ident = buffa_codegen::idents::make_field_ident(field_name);
                field_checks.extend(constraints::message::generate_nested_validation(
                    &field_ident,
                    field_name,
                    true,
                ));
            }
        }
    }

    // Validate repeated items (message types that implement Validate)
    for field_desc in &message.field {
        let field_name = field_desc.name.as_deref().unwrap_or("");
        let is_message_type = field_desc.r#type
            == Some(
                buffa_codegen::generated::descriptor::field_descriptor_proto::Type::TYPE_MESSAGE,
            );

        if let Some(field_rules) = rules::field_rules(field_desc) {
            if let Some(crate::generated::TypeRules::Repeated(_)) = field_rules.type_rules
                && is_message_type {
                    let field_type_name = field_desc.type_name.as_deref().unwrap_or("");
                    if !field_type_name.starts_with(".google.protobuf.") {
                        has_any_rules = true;
                        let field_ident = buffa_codegen::idents::make_field_ident(field_name);
                        field_checks.extend(quote! {
                            for (__idx, __item) in self.#field_ident.iter().enumerate() {
                                if let ::core::result::Result::Err(nested_violations) = ::buffa_validate::Validate::validate(__item) {
                                    for mut v in nested_violations.violations {
                                        v.field_path = ::std::format!("{}[{}].{}", #field_name, __idx, v.field_path);
                                        violations.push(v);
                                    }
                                }
                            }
                        });
                    }
                }
            if let Some(crate::generated::TypeRules::Map(ref map_rules)) = field_rules.type_rules
                && (map_rules.keys.is_some() || map_rules.values.is_some()) {
                    let field_ident = buffa_codegen::idents::make_field_ident(field_name);
                    let mut entry_checks = TokenStream::new();

                    if let Some(ref keys_rules) = map_rules.keys
                        && let Some(ref type_rules) = keys_rules.type_rules {
                            let key_checks = generate_map_key_checks(type_rules, field_name)?;
                            entry_checks.extend(key_checks);
                        }
                    if let Some(ref values_rules) = map_rules.values
                        && let Some(ref type_rules) = values_rules.type_rules {
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
    }

    // Recurse into nested messages
    for nested in &message.nested_type {
        let nested_name = nested.name.as_deref().unwrap_or("");
        let nested_fqn = format!("{fqn}.{nested_name}");
        generate_message_validation(nested, &nested_fqn, package, ctx, out)?;
    }

    Ok(())
}

use crate::generated::TypeRules;

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
        _ => Ok(TokenStream::new()),
    }
}
