use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use crate::generated::TimestampRules;

pub fn generate(rules: &TimestampRules, field_path: &str) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    if let Some(c) = &rules.r#const {
        let s = c.seconds;
        let n = c.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_timestamp_const(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "timestamp.const", msg));
            }
        });
    }

    if let Some(lt) = &rules.lt {
        let s = lt.seconds;
        let n = lt.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_timestamp_lt(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "timestamp.lt", msg));
            }
        });
    }

    if let Some(lte) = &rules.lte {
        let s = lte.seconds;
        let n = lte.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_timestamp_lte(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "timestamp.lte", msg));
            }
        });
    }

    if let Some(gt) = &rules.gt {
        let s = gt.seconds;
        let n = gt.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_timestamp_gt(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "timestamp.gt", msg));
            }
        });
    }

    if let Some(gte) = &rules.gte {
        let s = gte.seconds;
        let n = gte.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_timestamp_gte(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "timestamp.gte", msg));
            }
        });
    }

    if rules.lt_now {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_timestamp_lt_now(__secs, __nanos) {
                violations.push(::buffa_validate::Violation::new(#field_path, "timestamp.lt_now", msg));
            }
        });
    }

    if rules.gt_now {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_timestamp_gt_now(__secs, __nanos) {
                violations.push(::buffa_validate::Violation::new(#field_path, "timestamp.gt_now", msg));
            }
        });
    }

    if let Some(w) = &rules.within {
        let s = w.seconds;
        let n = w.nanos;
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_timestamp_within(__secs, __nanos, #s, #n) {
                violations.push(::buffa_validate::Violation::new(#field_path, "timestamp.within", msg));
            }
        });
    }

    Ok(checks)
}
