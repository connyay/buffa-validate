use anyhow::Result;
use buffa_codegen::context::CodeGenContext;
use buffa_codegen::generated::descriptor::{DescriptorProto, FileDescriptorProto};
use proc_macro2::TokenStream;
use quote::quote;

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

    // Check field-level rules
    for field_desc in &message.field {
        let field_name = field_desc.name.as_deref().unwrap_or("");

        if let Some(field_rules) = rules::field_rules(field_desc) {
            has_any_rules = true;
            let check =
                field::generate_field_validation(field_desc, &field_rules, field_name, fqn, ctx)?;
            field_checks.extend(check);
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
