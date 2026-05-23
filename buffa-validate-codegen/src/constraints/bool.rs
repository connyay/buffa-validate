use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use crate::generated::BoolRules;

pub fn generate(rules: &BoolRules, field_path: &str) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    if let Some(expected) = rules.r#const {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_bool_const(__field_val, #expected) {
                violations.push(::buffa_validate::Violation::new(#field_path, "bool.const", msg));
            }
        });
    }

    Ok(checks)
}
