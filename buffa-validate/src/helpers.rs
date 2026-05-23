use std::sync::OnceLock;

use regex::Regex;

pub fn check_string_min_len(val: &str, min: u64) -> Option<String> {
    let len = val.chars().count() as u64;
    if len < min {
        Some(format!("value length must be at least {min} characters"))
    } else {
        None
    }
}

pub fn check_string_max_len(val: &str, max: u64) -> Option<String> {
    let len = val.chars().count() as u64;
    if len > max {
        Some(format!("value length must be at most {max} characters"))
    } else {
        None
    }
}

pub fn check_string_len(val: &str, expected: u64) -> Option<String> {
    let len = val.chars().count() as u64;
    if len != expected {
        Some(format!(
            "value length must be exactly {expected} characters"
        ))
    } else {
        None
    }
}

pub fn check_string_min_bytes(val: &str, min: u64) -> Option<String> {
    if (val.len() as u64) < min {
        Some(format!("value byte length must be at least {min}"))
    } else {
        None
    }
}

pub fn check_string_max_bytes(val: &str, max: u64) -> Option<String> {
    if (val.len() as u64) > max {
        Some(format!("value byte length must be at most {max}"))
    } else {
        None
    }
}

pub fn check_string_pattern(val: &str, pattern: &OnceLock<Regex>, raw: &str) -> Option<String> {
    let re = pattern.get_or_init(|| Regex::new(raw).expect("invalid regex pattern in proto"));
    if !re.is_match(val) {
        Some(format!("value must match pattern '{raw}'"))
    } else {
        None
    }
}

pub fn check_string_prefix(val: &str, prefix: &str) -> Option<String> {
    if !val.starts_with(prefix) {
        Some(format!("value must have prefix '{prefix}'"))
    } else {
        None
    }
}

pub fn check_string_suffix(val: &str, suffix: &str) -> Option<String> {
    if !val.ends_with(suffix) {
        Some(format!("value must have suffix '{suffix}'"))
    } else {
        None
    }
}

pub fn check_string_contains(val: &str, needle: &str) -> Option<String> {
    if !val.contains(needle) {
        Some(format!("value must contain '{needle}'"))
    } else {
        None
    }
}

pub fn check_string_not_contains(val: &str, needle: &str) -> Option<String> {
    if val.contains(needle) {
        Some(format!("value must not contain '{needle}'"))
    } else {
        None
    }
}

pub fn check_string_const(val: &str, expected: &str) -> Option<String> {
    if val != expected {
        Some(format!("value must equal '{expected}'"))
    } else {
        None
    }
}

pub fn check_string_in(val: &str, set: &[&str]) -> Option<String> {
    if !set.contains(&val) {
        Some("value must be in the allowed set".to_string())
    } else {
        None
    }
}

pub fn check_string_not_in(val: &str, set: &[&str]) -> Option<String> {
    if set.contains(&val) {
        Some("value must not be in the denied set".to_string())
    } else {
        None
    }
}

static EMAIL_RE: OnceLock<Regex> = OnceLock::new();

pub fn check_string_email(val: &str) -> Option<String> {
    if val.is_empty() {
        return Some("value must be a valid email address".to_string());
    }
    let re =
        EMAIL_RE.get_or_init(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").expect("email regex"));
    if !re.is_match(val) {
        Some("value must be a valid email address".to_string())
    } else {
        None
    }
}

pub fn check_string_uuid(val: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
            .expect("uuid regex")
    });
    if !re.is_match(val) {
        Some("value must be a valid UUID".to_string())
    } else {
        None
    }
}

pub fn check_string_hostname(val: &str) -> Option<String> {
    if val.is_empty() || val.len() > 253 {
        return Some("value must be a valid hostname".to_string());
    }
    for label in val.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Some("value must be a valid hostname".to_string());
        }
    }
    None
}

pub fn check_string_ip(val: &str) -> Option<String> {
    if val.parse::<std::net::IpAddr>().is_err() {
        Some("value must be a valid IP address".to_string())
    } else {
        None
    }
}

pub fn check_string_ipv4(val: &str) -> Option<String> {
    if val.parse::<std::net::Ipv4Addr>().is_err() {
        Some("value must be a valid IPv4 address".to_string())
    } else {
        None
    }
}

pub fn check_string_ipv6(val: &str) -> Option<String> {
    if val.parse::<std::net::Ipv6Addr>().is_err() {
        Some("value must be a valid IPv6 address".to_string())
    } else {
        None
    }
}

pub fn check_string_uri(val: &str) -> Option<String> {
    if val.is_empty() || !val.contains(':') {
        return Some("value must be a valid URI".to_string());
    }
    let scheme_end = val.find(':').unwrap();
    let scheme = &val[..scheme_end];
    if scheme.is_empty()
        || !scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    {
        return Some("value must be a valid URI".to_string());
    }
    None
}

// Numeric helpers

pub fn check_gt<T: PartialOrd + std::fmt::Display>(val: T, bound: T) -> Option<String> {
    if val <= bound {
        Some(format!("value must be greater than {bound}"))
    } else {
        None
    }
}

pub fn check_gte<T: PartialOrd + std::fmt::Display>(val: T, bound: T) -> Option<String> {
    if val < bound {
        Some(format!("value must be greater than or equal to {bound}"))
    } else {
        None
    }
}

pub fn check_lt<T: PartialOrd + std::fmt::Display>(val: T, bound: T) -> Option<String> {
    if val >= bound {
        Some(format!("value must be less than {bound}"))
    } else {
        None
    }
}

pub fn check_lte<T: PartialOrd + std::fmt::Display>(val: T, bound: T) -> Option<String> {
    if val > bound {
        Some(format!("value must be less than or equal to {bound}"))
    } else {
        None
    }
}

pub fn check_const<T: PartialEq + std::fmt::Display>(val: T, expected: T) -> Option<String> {
    if val != expected {
        Some(format!("value must equal {expected}"))
    } else {
        None
    }
}

pub fn check_in<T: PartialEq>(val: &T, set: &[T]) -> Option<String> {
    if !set.contains(val) {
        Some("value must be in the allowed set".to_string())
    } else {
        None
    }
}

pub fn check_not_in<T: PartialEq>(val: &T, set: &[T]) -> Option<String> {
    if set.contains(val) {
        Some("value must not be in the denied set".to_string())
    } else {
        None
    }
}

pub fn check_repeated_min_items(len: usize, min: u64) -> Option<String> {
    if (len as u64) < min {
        Some(format!("value must have at least {min} items"))
    } else {
        None
    }
}

pub fn check_repeated_max_items(len: usize, max: u64) -> Option<String> {
    if (len as u64) > max {
        Some(format!("value must have at most {max} items"))
    } else {
        None
    }
}

pub fn check_repeated_unique<T: Eq + std::hash::Hash>(items: &[T]) -> Option<String> {
    let mut seen = std::collections::HashSet::with_capacity(items.len());
    for item in items {
        if !seen.insert(item) {
            return Some("repeated value must contain unique items".to_string());
        }
    }
    None
}

pub fn check_map_min_pairs(len: usize, min: u64) -> Option<String> {
    if (len as u64) < min {
        Some(format!("map must have at least {min} entries"))
    } else {
        None
    }
}

pub fn check_map_max_pairs(len: usize, max: u64) -> Option<String> {
    if (len as u64) > max {
        Some(format!("map must have at most {max} entries"))
    } else {
        None
    }
}

pub fn check_bool_const(val: bool, expected: bool) -> Option<String> {
    if val != expected {
        Some(format!("value must be {expected}"))
    } else {
        None
    }
}

pub fn check_enum_defined_only(val: i32, known: &[i32]) -> Option<String> {
    if !known.contains(&val) {
        Some(format!("value ({val}) must be a defined enum value"))
    } else {
        None
    }
}

pub fn check_float_finite(val: f64) -> Option<String> {
    if !val.is_finite() {
        Some("value must be finite".to_string())
    } else {
        None
    }
}
