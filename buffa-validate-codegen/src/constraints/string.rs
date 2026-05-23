use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

use crate::generated::{StringRules, StringWellKnown};

pub fn generate(rules: &StringRules, field_path: &str) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    if let Some(ref c) = rules.r#const {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_const(__field_val, #c) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.const", msg));
            }
        });
    }

    if let Some(len) = rules.len {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_len(__field_val, #len) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.len", msg));
            }
        });
    }

    if let Some(min) = rules.min_len {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_min_len(__field_val, #min) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.min_len", msg));
            }
        });
    }

    if let Some(max) = rules.max_len {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_max_len(__field_val, #max) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.max_len", msg));
            }
        });
    }

    if let Some(len) = rules.len_bytes {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_len_bytes(__field_val, #len) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.len_bytes", msg));
            }
        });
    }

    if let Some(min) = rules.min_bytes {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_min_bytes(__field_val, #min) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.min_bytes", msg));
            }
        });
    }

    if let Some(max) = rules.max_bytes {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_max_bytes(__field_val, #max) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.max_bytes", msg));
            }
        });
    }

    if let Some(ref pattern) = rules.pattern {
        checks.extend(quote! {
            {
                static __RE: ::std::sync::OnceLock<::buffa_validate::__private::Regex> = ::std::sync::OnceLock::new();
                if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_pattern(__field_val, &__RE, #pattern) {
                    violations.push(::buffa_validate::Violation::new(#field_path, "string.pattern", msg));
                }
            }
        });
    }

    if let Some(ref prefix) = rules.prefix {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_prefix(__field_val, #prefix) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.prefix", msg));
            }
        });
    }

    if let Some(ref suffix) = rules.suffix {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_suffix(__field_val, #suffix) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.suffix", msg));
            }
        });
    }

    if let Some(ref contains) = rules.contains {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_contains(__field_val, #contains) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.contains", msg));
            }
        });
    }

    if let Some(ref not_contains) = rules.not_contains {
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_not_contains(__field_val, #not_contains) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.not_contains", msg));
            }
        });
    }

    if !rules.r#in.is_empty() {
        let values: Vec<&str> = rules.r#in.iter().map(|s| s.as_str()).collect();
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_in(__field_val, &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.in", msg));
            }
        });
    }

    if !rules.not_in.is_empty() {
        let values: Vec<&str> = rules.not_in.iter().map(|s| s.as_str()).collect();
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::check_string_not_in(__field_val, &[#(#values),*]) {
                violations.push(::buffa_validate::Violation::new(#field_path, "string.not_in", msg));
            }
        });
    }

    if let Some(ref wk) = rules.well_known {
        let (helper_fn, rule_id) = match wk {
            StringWellKnown::Email => ("check_string_email", "string.email"),
            StringWellKnown::Hostname => ("check_string_hostname", "string.hostname"),
            StringWellKnown::Ip => ("check_string_ip", "string.ip"),
            StringWellKnown::Ipv4 => ("check_string_ipv4", "string.ipv4"),
            StringWellKnown::Ipv6 => ("check_string_ipv6", "string.ipv6"),
            StringWellKnown::Uri | StringWellKnown::UriRef => ("check_string_uri", "string.uri"),
            StringWellKnown::Uuid | StringWellKnown::Tuuid => ("check_string_uuid", "string.uuid"),
            StringWellKnown::Address => ("check_string_address", "string.address"),
            StringWellKnown::IpWithPrefixlen => {
                ("check_string_ip_with_prefixlen", "string.ip_with_prefixlen")
            }
            StringWellKnown::Ipv4WithPrefixlen => (
                "check_string_ipv4_with_prefixlen",
                "string.ipv4_with_prefixlen",
            ),
            StringWellKnown::Ipv6WithPrefixlen => (
                "check_string_ipv6_with_prefixlen",
                "string.ipv6_with_prefixlen",
            ),
            StringWellKnown::IpPrefix => ("check_string_ip_prefix", "string.ip_prefix"),
            StringWellKnown::Ipv4Prefix => ("check_string_ipv4_prefix", "string.ipv4_prefix"),
            StringWellKnown::Ipv6Prefix => ("check_string_ipv6_prefix", "string.ipv6_prefix"),
            StringWellKnown::HostAndPort => ("check_string_host_and_port", "string.host_and_port"),
            StringWellKnown::WellKnownRegex(_) => return Ok(checks),
        };
        let helper_ident = proc_macro2::Ident::new(helper_fn, proc_macro2::Span::call_site());
        checks.extend(quote! {
            if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::#helper_ident(__field_val) {
                violations.push(::buffa_validate::Violation::new(#field_path, #rule_id, msg));
            }
        });
    }

    Ok(checks)
}
