use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use crate::generated::BytesRules;

pub fn generate(rules: &BytesRules, field_path: &str) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    if let Some(ref c) = rules.r#const {
        let bytes = c.as_slice();
        checks.extend(quote! {
            if __field_val != &[#(#bytes),*] {
                violations.push(::buffa_validate::Violation::new(#field_path, "bytes.const", "value must equal the specified bytes"));
            }
        });
    }

    if let Some(len) = rules.len {
        let len_usize = len as usize;
        checks.extend(quote! {
            if __field_val.len() != #len_usize {
                violations.push(::buffa_validate::Violation::new(
                    #field_path,
                    "bytes.len",
                    ::std::format!("value byte length must be exactly {}", #len),
                ));
            }
        });
    }

    if let Some(min) = rules.min_len {
        let min_usize = min as usize;
        checks.extend(quote! {
            if __field_val.len() < #min_usize {
                violations.push(::buffa_validate::Violation::new(
                    #field_path,
                    "bytes.min_len",
                    ::std::format!("value byte length must be at least {}", #min),
                ));
            }
        });
    }

    if let Some(max) = rules.max_len {
        let max_usize = max as usize;
        checks.extend(quote! {
            if __field_val.len() > #max_usize {
                violations.push(::buffa_validate::Violation::new(
                    #field_path,
                    "bytes.max_len",
                    ::std::format!("value byte length must be at most {}", #max),
                ));
            }
        });
    }

    if let Some(ref prefix) = rules.prefix {
        let bytes = prefix.as_slice();
        checks.extend(quote! {
            if !__field_val.starts_with(&[#(#bytes),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "bytes.prefix", "value must have the specified prefix"));
            }
        });
    }

    if let Some(ref suffix) = rules.suffix {
        let bytes = suffix.as_slice();
        checks.extend(quote! {
            if !__field_val.ends_with(&[#(#bytes),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "bytes.suffix", "value must have the specified suffix"));
            }
        });
    }

    Ok(checks)
}
