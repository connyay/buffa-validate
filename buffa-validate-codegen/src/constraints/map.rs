use anyhow::Result;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::generated::MapRules;

pub fn generate(rules: &MapRules, field_ident: &Ident, field_path: &str) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    if let Some(min) = rules.min_pairs {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_map_min_pairs(self.#field_ident.len(), #min) {
                violations.push(::buffa_validate::Violation::new(#field_path, "map.min_pairs", msg));
            }
        });
    }

    if let Some(max) = rules.max_pairs {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_map_max_pairs(self.#field_ident.len(), #max) {
                violations.push(::buffa_validate::Violation::new(#field_path, "map.max_pairs", msg));
            }
        });
    }

    Ok(checks)
}
