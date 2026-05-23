use anyhow::Result;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::generated::RepeatedRules;

pub fn generate(
    rules: &RepeatedRules,
    field_ident: &Ident,
    field_path: &str,
) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    if let Some(min) = rules.min_items {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_repeated_min_items(self.#field_ident.len(), #min) {
                violations.push(::buffa_validate::Violation::new(#field_path, "repeated.min_items", msg));
            }
        });
    }

    if let Some(max) = rules.max_items {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_repeated_max_items(self.#field_ident.len(), #max) {
                violations.push(::buffa_validate::Violation::new(#field_path, "repeated.max_items", msg));
            }
        });
    }

    if rules.unique {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_repeated_unique(&self.#field_ident) {
                violations.push(::buffa_validate::Violation::new(#field_path, "repeated.unique", msg));
            }
        });
    }

    Ok(checks)
}
