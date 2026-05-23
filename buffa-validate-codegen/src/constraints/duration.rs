use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use crate::generated::DurationRules;

pub fn generate(rules: &DurationRules, field_path: &str) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    if let Some(c) = &rules.r#const {
        let s = c.seconds;
        let n = c.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_duration_const(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "duration.const", msg));
            }
        });
    }

    if let Some(lt) = &rules.lt {
        let s = lt.seconds;
        let n = lt.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_duration_lt(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "duration.lt", msg));
            }
        });
    }

    if let Some(lte) = &rules.lte {
        let s = lte.seconds;
        let n = lte.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_duration_lte(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "duration.lte", msg));
            }
        });
    }

    if let Some(gt) = &rules.gt {
        let s = gt.seconds;
        let n = gt.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_duration_gt(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "duration.gt", msg));
            }
        });
    }

    if let Some(gte) = &rules.gte {
        let s = gte.seconds;
        let n = gte.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_duration_gte(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "duration.gte", msg));
            }
        });
    }

    if !rules.r#in.is_empty() {
        let pairs: Vec<TokenStream> = rules
            .r#in
            .iter()
            .map(|tv| {
                let s = tv.seconds;
                let n = tv.nanos;
                quote! { (#s, #n) }
            })
            .collect();
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_duration_in(__secs, __nanos, &[#(#pairs),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "duration.in", msg));
            }
        });
    }

    if !rules.not_in.is_empty() {
        let pairs: Vec<TokenStream> = rules
            .not_in
            .iter()
            .map(|tv| {
                let s = tv.seconds;
                let n = tv.nanos;
                quote! { (#s, #n) }
            })
            .collect();
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_duration_not_in(__secs, __nanos, &[#(#pairs),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "duration.not_in", msg));
            }
        });
    }

    Ok(checks)
}
