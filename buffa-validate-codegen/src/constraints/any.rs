use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use crate::generated::AnyRules;

pub fn generate(rules: &AnyRules, field_path: &str) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    if !rules.r#in.is_empty() {
        let values: Vec<&str> = rules.r#in.iter().map(|s| s.as_str()).collect();
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_any_in(__type_url, &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "any.in", msg));
            }
        });
    }

    if !rules.not_in.is_empty() {
        let values: Vec<&str> = rules.not_in.iter().map(|s| s.as_str()).collect();
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_any_not_in(__type_url, &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "any.not_in", msg));
            }
        });
    }

    Ok(checks)
}
