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

pub fn check_string_len_bytes(val: &str, expected: u64) -> Option<String> {
    if (val.len() as u64) != expected {
        Some(format!("value byte length must be exactly {expected}"))
    } else {
        None
    }
}

pub fn bytes_contains(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

pub fn bytes_in(val: &[u8], set: &[&[u8]]) -> bool {
    set.contains(&val)
}

pub fn check_string_address(val: &str) -> Option<String> {
    if val.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }
    check_string_hostname(val)?;
    Some("value must be a valid hostname or IP address".to_string())
}

pub fn check_string_ip_with_prefixlen(val: &str) -> Option<String> {
    check_ip_prefix_common(val, false, None)
}

pub fn check_string_ipv4_with_prefixlen(val: &str) -> Option<String> {
    check_ip_prefix_common(val, false, Some(4))
}

pub fn check_string_ipv6_with_prefixlen(val: &str) -> Option<String> {
    check_ip_prefix_common(val, false, Some(6))
}

pub fn check_string_ip_prefix(val: &str) -> Option<String> {
    check_ip_prefix_common(val, true, None)
}

pub fn check_string_ipv4_prefix(val: &str) -> Option<String> {
    check_ip_prefix_common(val, true, Some(4))
}

pub fn check_string_ipv6_prefix(val: &str) -> Option<String> {
    check_ip_prefix_common(val, true, Some(6))
}

fn check_ip_prefix_common(val: &str, must_be_prefix: bool, version: Option<u8>) -> Option<String> {
    let Some((ip_str, prefix_str)) = val.split_once('/') else {
        return Some("value must be a valid IP prefix (CIDR notation)".to_string());
    };
    let Ok(prefix_len) = prefix_str.parse::<u32>() else {
        return Some("value must have a valid prefix length".to_string());
    };
    match ip_str.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(addr)) => {
            if version == Some(6) {
                return Some("value must be an IPv6 prefix".to_string());
            }
            if prefix_len > 32 {
                return Some("IPv4 prefix length must be <= 32".to_string());
            }
            if must_be_prefix {
                let bits = u32::from(addr);
                let host_bits = 32 - prefix_len;
                if host_bits < 32 && (bits & ((1u32 << host_bits) - 1)) != 0 {
                    return Some("value must be a network prefix, not a host address".to_string());
                }
            }
            None
        }
        Ok(std::net::IpAddr::V6(addr)) => {
            if version == Some(4) {
                return Some("value must be an IPv4 prefix".to_string());
            }
            if prefix_len > 128 {
                return Some("IPv6 prefix length must be <= 128".to_string());
            }
            if must_be_prefix {
                let bits = u128::from(addr);
                let host_bits = 128 - prefix_len;
                if host_bits < 128 && (bits & ((1u128 << host_bits) - 1)) != 0 {
                    return Some("value must be a network prefix, not a host address".to_string());
                }
            }
            None
        }
        Err(_) => Some("value must contain a valid IP address".to_string()),
    }
}

pub fn check_string_host_and_port(val: &str) -> Option<String> {
    if let Some(rest) = val.strip_prefix('[') {
        let Some((_, port_str)) = rest.rsplit_once("]:") else {
            return Some("value must be a valid host:port pair".to_string());
        };
        if port_str.parse::<u16>().is_err() {
            return Some("value must have a valid port number".to_string());
        }
        return None;
    }
    let Some((host, port_str)) = val.rsplit_once(':') else {
        return Some("value must be a valid host:port pair".to_string());
    };
    if host.is_empty() || port_str.parse::<u16>().is_err() {
        return Some("value must be a valid host:port pair".to_string());
    }
    None
}

// CEL helpers

pub fn cel_compile(source: &str) -> cel::Program {
    cel::Program::compile(source)
        .unwrap_or_else(|e| panic!("invalid CEL expression: {source}: {e}"))
}

pub fn cel_check(
    program: &cel::Program,
    ctx: &cel::Context,
    rule_id: &str,
    default_message: &str,
) -> Option<String> {
    match program.execute(ctx) {
        Ok(cel::Value::Bool(true)) => None,
        Ok(cel::Value::Bool(false)) => Some(default_message.to_string()),
        Ok(cel::Value::String(msg)) => {
            if msg.is_empty() {
                None
            } else {
                Some(msg.to_string())
            }
        }
        Ok(_) => Some(format!(
            "CEL rule '{rule_id}' returned non-bool/non-string value"
        )),
        Err(e) => Some(format!("CEL rule '{rule_id}' execution error: {e}")),
    }
}

pub fn field_to_cel_value_string(val: &str) -> cel::Value {
    cel::Value::String(std::sync::Arc::new(val.to_string()))
}

pub fn field_to_cel_value_bytes(val: &[u8]) -> cel::Value {
    cel::Value::Bytes(std::sync::Arc::new(val.to_vec()))
}

pub fn field_to_cel_value_bool(val: bool) -> cel::Value {
    cel::Value::Bool(val)
}

pub fn field_to_cel_value_int(val: i64) -> cel::Value {
    cel::Value::Int(val)
}

pub fn field_to_cel_value_uint(val: u64) -> cel::Value {
    cel::Value::UInt(val)
}

pub fn field_to_cel_value_float(val: f64) -> cel::Value {
    cel::Value::Float(val)
}
