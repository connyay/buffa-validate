use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use crate::generated::EnumRules;

pub fn generate(rules: &EnumRules, field_path: &str) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    if let Some(c) = rules.r#const {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_const(__field_val.to_i32(), #c) {
                violations.push(::buffa_validate::Violation::new(#field_path, "enum.const", msg));
            }
        });
    }

    if rules.defined_only {
        // defined_only is handled via the EnumValue type at runtime
        checks.extend(quote! {
            if __field_val.is_unknown() {
                violations.push(::buffa_validate::Violation::new(
                    #field_path,
                    "enum.defined_only",
                    "value must be a defined enum value",
                ));
            }
        });
    }

    if !rules.r#in.is_empty() {
        let values = &rules.r#in;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_in(&__field_val.to_i32(), &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "enum.in", msg));
            }
        });
    }

    if !rules.not_in.is_empty() {
        let values = &rules.not_in;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_not_in(&__field_val.to_i32(), &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "enum.not_in", msg));
            }
        });
    }

    Ok(checks)
}
