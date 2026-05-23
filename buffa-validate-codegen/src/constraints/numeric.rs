use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use crate::generated::NumericRules;

pub fn generate_int<T: quote::ToTokens + std::fmt::Display>(
    rules: &NumericRules<T>,
    field_path: &str,
    _type_name: &str,
) -> Result<TokenStream> {
    generate_common(rules, field_path)
}

pub fn generate_uint<T: quote::ToTokens + std::fmt::Display>(
    rules: &NumericRules<T>,
    field_path: &str,
    _type_name: &str,
) -> Result<TokenStream> {
    generate_common(rules, field_path)
}

pub fn generate_float(rules: &NumericRules<f32>, field_path: &str) -> Result<TokenStream> {
    let mut checks = generate_common(rules, field_path)?;
    if rules.finite {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_float_finite(__field_val as f64) {
                violations.push(::buffa_validate::Violation::new(#field_path, "float.finite", msg));
            }
        });
    }
    Ok(checks)
}

pub fn generate_double(rules: &NumericRules<f64>, field_path: &str) -> Result<TokenStream> {
    let mut checks = generate_common(rules, field_path)?;
    if rules.finite {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_float_finite(__field_val) {
                violations.push(::buffa_validate::Violation::new(#field_path, "double.finite", msg));
            }
        });
    }
    Ok(checks)
}

fn generate_common<T: quote::ToTokens + std::fmt::Display>(
    rules: &NumericRules<T>,
    field_path: &str,
) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    if let Some(ref c) = rules.r#const {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_const(__field_val, #c) {
                violations.push(::buffa_validate::Violation::new(#field_path, "const", msg));
            }
        });
    }

    if let Some(ref bound) = rules.gt {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_gt(__field_val, #bound) {
                violations.push(::buffa_validate::Violation::new(#field_path, "gt", msg));
            }
        });
    }

    if let Some(ref bound) = rules.gte {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_gte(__field_val, #bound) {
                violations.push(::buffa_validate::Violation::new(#field_path, "gte", msg));
            }
        });
    }

    if let Some(ref bound) = rules.lt {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_lt(__field_val, #bound) {
                violations.push(::buffa_validate::Violation::new(#field_path, "lt", msg));
            }
        });
    }

    if let Some(ref bound) = rules.lte {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_lte(__field_val, #bound) {
                violations.push(::buffa_validate::Violation::new(#field_path, "lte", msg));
            }
        });
    }

    if !rules.r#in.is_empty() {
        let values = &rules.r#in;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_in(&__field_val, &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "in", msg));
            }
        });
    }

    if !rules.not_in.is_empty() {
        let values = &rules.not_in;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_not_in(&__field_val, &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "not_in", msg));
            }
        });
    }

    Ok(checks)
}
