use buffa::{UnknownFieldData, UnknownFields};

const FIELD_RULES_EXT_NUMBER: u32 = 1159;
const MESSAGE_RULES_EXT_NUMBER: u32 = 1159;
const ONEOF_RULES_EXT_NUMBER: u32 = 1159;

pub fn extract_field_rules(unknown_fields: &UnknownFields) -> Option<FieldRules> {
    for field in unknown_fields.iter() {
        if field.number == FIELD_RULES_EXT_NUMBER
            && let UnknownFieldData::LengthDelimited(bytes) = &field.data
        {
            return FieldRules::decode_from_slice(bytes).ok();
        }
    }
    None
}

pub fn extract_message_rules(unknown_fields: &UnknownFields) -> Option<MessageRules> {
    for field in unknown_fields.iter() {
        if field.number == MESSAGE_RULES_EXT_NUMBER
            && let UnknownFieldData::LengthDelimited(bytes) = &field.data
        {
            return MessageRules::decode_from_slice(bytes).ok();
        }
    }
    None
}

pub fn extract_oneof_rules(unknown_fields: &UnknownFields) -> Option<OneofRules> {
    for field in unknown_fields.iter() {
        if field.number == ONEOF_RULES_EXT_NUMBER
            && let UnknownFieldData::LengthDelimited(bytes) = &field.data
        {
            return OneofRules::decode_from_slice(bytes).ok();
        }
    }
    None
}

#[derive(Debug, Clone, Default)]
pub struct FieldRules {
    pub cel: Vec<Rule>,
    pub required: bool,
    pub ignore: Ignore,
    pub type_rules: Option<TypeRules>,
}

#[derive(Debug, Clone)]
pub enum TypeRules {
    Float(NumericRules<f32>),
    Double(NumericRules<f64>),
    Int32(NumericRules<i32>),
    Int64(NumericRules<i64>),
    Uint32(NumericRules<u32>),
    Uint64(NumericRules<u64>),
    Sint32(NumericRules<i32>),
    Sint64(NumericRules<i64>),
    Fixed32(NumericRules<u32>),
    Fixed64(NumericRules<u64>),
    Sfixed32(NumericRules<i32>),
    Sfixed64(NumericRules<i64>),
    Bool(BoolRules),
    String(StringRules),
    Bytes(BytesRules),
    Enum(EnumRules),
    Repeated(RepeatedRules),
    Map(MapRules),
    Any(AnyRules),
    Duration(DurationRules),
    Timestamp(TimestampRules),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProtoTimeval {
    pub seconds: i64,
    pub nanos: i32,
}

#[derive(Debug, Clone, Default)]
pub struct AnyRules {
    pub r#in: Vec<String>,
    pub not_in: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DurationRules {
    pub r#const: Option<ProtoTimeval>,
    pub lt: Option<ProtoTimeval>,
    pub lte: Option<ProtoTimeval>,
    pub gt: Option<ProtoTimeval>,
    pub gte: Option<ProtoTimeval>,
    pub r#in: Vec<ProtoTimeval>,
    pub not_in: Vec<ProtoTimeval>,
}

#[derive(Debug, Clone, Default)]
pub struct TimestampRules {
    pub r#const: Option<ProtoTimeval>,
    pub lt: Option<ProtoTimeval>,
    pub lte: Option<ProtoTimeval>,
    pub lt_now: bool,
    pub gt: Option<ProtoTimeval>,
    pub gte: Option<ProtoTimeval>,
    pub gt_now: bool,
    pub within: Option<ProtoTimeval>,
}

#[derive(Debug, Clone, Default)]
pub struct MessageRules {
    pub cel: Vec<Rule>,
}

#[derive(Debug, Clone, Default)]
pub struct OneofRules {
    pub required: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Rule {
    pub id: Option<String>,
    pub message: Option<String>,
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Ignore {
    #[default]
    Unspecified,
    IfDefaultValue,
    Always,
}

#[derive(Debug, Clone, Default)]
pub struct StringRules {
    pub r#const: Option<String>,
    pub len: Option<u64>,
    pub min_len: Option<u64>,
    pub max_len: Option<u64>,
    pub len_bytes: Option<u64>,
    pub min_bytes: Option<u64>,
    pub max_bytes: Option<u64>,
    pub pattern: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub contains: Option<String>,
    pub not_contains: Option<String>,
    pub r#in: Vec<String>,
    pub not_in: Vec<String>,
    pub well_known: Option<StringWellKnown>,
    pub strict: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum StringWellKnown {
    Email,
    Hostname,
    Ip,
    Ipv4,
    Ipv6,
    Uri,
    UriRef,
    Uuid,
    Tuuid,
    Address,
    IpWithPrefixlen,
    Ipv4WithPrefixlen,
    Ipv6WithPrefixlen,
    IpPrefix,
    Ipv4Prefix,
    Ipv6Prefix,
    HostAndPort,
    WellKnownRegex(i32),
}

#[derive(Debug, Clone, Default)]
pub struct NumericRules<T> {
    pub r#const: Option<T>,
    pub lt: Option<T>,
    pub lte: Option<T>,
    pub gt: Option<T>,
    pub gte: Option<T>,
    pub r#in: Vec<T>,
    pub not_in: Vec<T>,
    pub finite: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BoolRules {
    pub r#const: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct BytesRules {
    pub r#const: Option<Vec<u8>>,
    pub len: Option<u64>,
    pub min_len: Option<u64>,
    pub max_len: Option<u64>,
    pub pattern: Option<String>,
    pub prefix: Option<Vec<u8>>,
    pub suffix: Option<Vec<u8>>,
    pub contains: Option<Vec<u8>>,
    pub r#in: Vec<Vec<u8>>,
    pub not_in: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Default)]
pub struct EnumRules {
    pub r#const: Option<i32>,
    pub defined_only: bool,
    pub r#in: Vec<i32>,
    pub not_in: Vec<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct RepeatedRules {
    pub min_items: Option<u64>,
    pub max_items: Option<u64>,
    pub unique: bool,
    pub items: Option<Box<FieldRules>>,
}

#[derive(Debug, Clone, Default)]
pub struct MapRules {
    pub min_pairs: Option<u64>,
    pub max_pairs: Option<u64>,
    pub keys: Option<Box<FieldRules>>,
    pub values: Option<Box<FieldRules>>,
}

// Manual protobuf decoders

fn decode_varint(buf: &mut &[u8]) -> Option<u64> {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    loop {
        if buf.is_empty() {
            return None;
        }
        let byte = buf[0];
        *buf = &buf[1..];
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some(result);
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
}

fn decode_length_delimited<'a>(buf: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = decode_varint(buf)? as usize;
    if buf.len() < len {
        return None;
    }
    let data = &buf[..len];
    *buf = &buf[len..];
    Some(data)
}

fn decode_tag(buf: &mut &[u8]) -> Option<(u32, u8)> {
    let v = decode_varint(buf)?;
    let field_number = (v >> 3) as u32;
    let wire_type = (v & 0x7) as u8;
    Some((field_number, wire_type))
}

fn skip_field(buf: &mut &[u8], wire_type: u8) -> Option<()> {
    match wire_type {
        0 => {
            decode_varint(buf)?;
        }
        1 => {
            if buf.len() < 8 {
                return None;
            }
            *buf = &buf[8..];
        }
        2 => {
            decode_length_delimited(buf)?;
        }
        5 => {
            if buf.len() < 4 {
                return None;
            }
            *buf = &buf[4..];
        }
        _ => return None,
    }
    Some(())
}

fn decode_string(data: &[u8]) -> String {
    String::from_utf8_lossy(data).into_owned()
}

fn decode_i32_from_varint(v: u64) -> i32 {
    v as i32
}

fn decode_i64_from_varint(v: u64) -> i64 {
    v as i64
}

fn decode_sint32_from_varint(v: u64) -> i32 {
    let n = v as u32;
    ((n >> 1) as i32) ^ -((n & 1) as i32)
}

fn decode_sint64_from_varint(v: u64) -> i64 {
    ((v >> 1) as i64) ^ -((v & 1) as i64)
}

impl FieldRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = FieldRules::default();
        let mut buf = data;

        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                // cel = 23, repeated message
                (23, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad cel")?;
                    rules.cel.push(Rule::decode_from_slice(sub)?);
                }
                // required = 25, bool
                (25, 0) => {
                    rules.required = decode_varint(&mut buf).ok_or("bad required")? != 0;
                }
                // ignore = 27, enum
                (27, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad ignore")?;
                    rules.ignore = match v {
                        1 => Ignore::IfDefaultValue,
                        3 => Ignore::Always,
                        _ => Ignore::Unspecified,
                    };
                }
                // Type-specific rules (oneof, fields 1-22)
                (1, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad float")?;
                    rules.type_rules = Some(TypeRules::Float(NumericRules::decode_float(sub)?));
                }
                (2, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad double")?;
                    rules.type_rules = Some(TypeRules::Double(NumericRules::decode_double(sub)?));
                }
                (3, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad int32")?;
                    rules.type_rules =
                        Some(TypeRules::Int32(NumericRules::decode_varint_signed32(sub)?));
                }
                (4, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad int64")?;
                    rules.type_rules =
                        Some(TypeRules::Int64(NumericRules::decode_varint_signed64(sub)?));
                }
                (5, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad uint32")?;
                    rules.type_rules =
                        Some(TypeRules::Uint32(NumericRules::decode_varint_u32(sub)?));
                }
                (6, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad uint64")?;
                    rules.type_rules =
                        Some(TypeRules::Uint64(NumericRules::decode_varint_u64(sub)?));
                }
                (7, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad sint32")?;
                    rules.type_rules = Some(TypeRules::Sint32(NumericRules::decode_sint32(sub)?));
                }
                (8, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad sint64")?;
                    rules.type_rules = Some(TypeRules::Sint64(NumericRules::decode_sint64(sub)?));
                }
                (9, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad fixed32")?;
                    rules.type_rules = Some(TypeRules::Fixed32(NumericRules::decode_fixed32(sub)?));
                }
                (10, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad fixed64")?;
                    rules.type_rules = Some(TypeRules::Fixed64(NumericRules::decode_fixed64(sub)?));
                }
                (11, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad sfixed32")?;
                    rules.type_rules =
                        Some(TypeRules::Sfixed32(NumericRules::decode_sfixed32(sub)?));
                }
                (12, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad sfixed64")?;
                    rules.type_rules =
                        Some(TypeRules::Sfixed64(NumericRules::decode_sfixed64(sub)?));
                }
                (13, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad bool")?;
                    rules.type_rules = Some(TypeRules::Bool(BoolRules::decode_from_slice(sub)?));
                }
                (14, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad string")?;
                    rules.type_rules =
                        Some(TypeRules::String(StringRules::decode_from_slice(sub)?));
                }
                (15, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad bytes")?;
                    rules.type_rules = Some(TypeRules::Bytes(BytesRules::decode_from_slice(sub)?));
                }
                (16, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad enum")?;
                    rules.type_rules = Some(TypeRules::Enum(EnumRules::decode_from_slice(sub)?));
                }
                (18, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad repeated")?;
                    rules.type_rules =
                        Some(TypeRules::Repeated(RepeatedRules::decode_from_slice(sub)?));
                }
                (19, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad map")?;
                    rules.type_rules = Some(TypeRules::Map(MapRules::decode_from_slice(sub)?));
                }
                (20, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad any")?;
                    rules.type_rules = Some(TypeRules::Any(AnyRules::decode_from_slice(sub)?));
                }
                (21, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad duration")?;
                    rules.type_rules =
                        Some(TypeRules::Duration(DurationRules::decode_from_slice(sub)?));
                }
                (22, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad timestamp")?;
                    rules.type_rules = Some(TypeRules::Timestamp(
                        TimestampRules::decode_from_slice(sub)?,
                    ));
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl Rule {
    fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rule = Rule::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad id")?;
                    rule.id = Some(decode_string(s));
                }
                (2, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad message")?;
                    rule.message = Some(decode_string(s));
                }
                (3, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad expression")?;
                    rule.expression = Some(decode_string(s));
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rule)
    }
}

impl MessageRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = MessageRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (3, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad cel")?;
                    rules.cel.push(Rule::decode_from_slice(sub)?);
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl OneofRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = OneofRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 0) => {
                    rules.required = decode_varint(&mut buf).ok_or("bad required")? != 0;
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl StringRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = StringRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad const")?;
                    rules.r#const = Some(decode_string(s));
                }
                (19, 0) => {
                    rules.len = Some(decode_varint(&mut buf).ok_or("bad len")?);
                }
                (2, 0) => {
                    rules.min_len = Some(decode_varint(&mut buf).ok_or("bad min_len")?);
                }
                (3, 0) => {
                    rules.max_len = Some(decode_varint(&mut buf).ok_or("bad max_len")?);
                }
                (20, 0) => {
                    rules.len_bytes = Some(decode_varint(&mut buf).ok_or("bad len_bytes")?);
                }
                (4, 0) => {
                    rules.min_bytes = Some(decode_varint(&mut buf).ok_or("bad min_bytes")?);
                }
                (5, 0) => {
                    rules.max_bytes = Some(decode_varint(&mut buf).ok_or("bad max_bytes")?);
                }
                (6, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad pattern")?;
                    rules.pattern = Some(decode_string(s));
                }
                (7, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad prefix")?;
                    rules.prefix = Some(decode_string(s));
                }
                (8, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad suffix")?;
                    rules.suffix = Some(decode_string(s));
                }
                (9, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad contains")?;
                    rules.contains = Some(decode_string(s));
                }
                (23, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad not_contains")?;
                    rules.not_contains = Some(decode_string(s));
                }
                (10, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad in")?;
                    rules.r#in.push(decode_string(s));
                }
                (11, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad not_in")?;
                    rules.not_in.push(decode_string(s));
                }
                // Well-known string types (oneof, fields 12-18, 21-22, 24-28)
                (12, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Email);
                    }
                }
                (13, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Hostname);
                    }
                }
                (14, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Ip);
                    }
                }
                (15, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Ipv4);
                    }
                }
                (16, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Ipv6);
                    }
                }
                (17, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Uri);
                    }
                }
                (18, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::UriRef);
                    }
                }
                (21, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Address);
                    }
                }
                (22, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Uuid);
                    }
                }
                (33, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Tuuid);
                    }
                }
                (24, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    rules.well_known = Some(StringWellKnown::WellKnownRegex(v as i32));
                }
                (25, 0) => {
                    rules.strict = Some(decode_varint(&mut buf).ok_or("bad strict")? != 0);
                }
                (26, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::IpWithPrefixlen);
                    }
                }
                (27, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Ipv4WithPrefixlen);
                    }
                }
                (28, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Ipv6WithPrefixlen);
                    }
                }
                (29, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::IpPrefix);
                    }
                }
                (30, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Ipv4Prefix);
                    }
                }
                (31, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::Ipv6Prefix);
                    }
                }
                (32, 0) => {
                    let v = decode_varint(&mut buf).ok_or("bad wk")?;
                    if v != 0 {
                        rules.well_known = Some(StringWellKnown::HostAndPort);
                    }
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl BoolRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = BoolRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 0) => {
                    rules.r#const = Some(decode_varint(&mut buf).ok_or("bad const")? != 0);
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl EnumRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = EnumRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 0) => {
                    rules.r#const = Some(decode_varint(&mut buf).ok_or("bad const")? as i32);
                }
                (2, 0) => {
                    rules.defined_only = decode_varint(&mut buf).ok_or("bad defined_only")? != 0;
                }
                (3, 0) => {
                    rules
                        .r#in
                        .push(decode_varint(&mut buf).ok_or("bad in")? as i32);
                }
                (3, 2) => {
                    // packed repeated
                    let data = decode_length_delimited(&mut buf).ok_or("bad in packed")?;
                    let mut sub = data;
                    while !sub.is_empty() {
                        rules
                            .r#in
                            .push(decode_varint(&mut sub).ok_or("bad in item")? as i32);
                    }
                }
                (4, 0) => {
                    rules
                        .not_in
                        .push(decode_varint(&mut buf).ok_or("bad not_in")? as i32);
                }
                (4, 2) => {
                    let data = decode_length_delimited(&mut buf).ok_or("bad not_in packed")?;
                    let mut sub = data;
                    while !sub.is_empty() {
                        rules
                            .not_in
                            .push(decode_varint(&mut sub).ok_or("bad not_in item")? as i32);
                    }
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl BytesRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = BytesRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 2) => {
                    let d = decode_length_delimited(&mut buf).ok_or("bad const")?;
                    rules.r#const = Some(d.to_vec());
                }
                (13, 0) => {
                    rules.len = Some(decode_varint(&mut buf).ok_or("bad len")?);
                }
                (2, 0) => {
                    rules.min_len = Some(decode_varint(&mut buf).ok_or("bad min_len")?);
                }
                (3, 0) => {
                    rules.max_len = Some(decode_varint(&mut buf).ok_or("bad max_len")?);
                }
                (4, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad pattern")?;
                    rules.pattern = Some(decode_string(s));
                }
                (5, 2) => {
                    let d = decode_length_delimited(&mut buf).ok_or("bad prefix")?;
                    rules.prefix = Some(d.to_vec());
                }
                (6, 2) => {
                    let d = decode_length_delimited(&mut buf).ok_or("bad suffix")?;
                    rules.suffix = Some(d.to_vec());
                }
                (7, 2) => {
                    let d = decode_length_delimited(&mut buf).ok_or("bad contains")?;
                    rules.contains = Some(d.to_vec());
                }
                (8, 2) => {
                    let d = decode_length_delimited(&mut buf).ok_or("bad in")?;
                    rules.r#in.push(d.to_vec());
                }
                (9, 2) => {
                    let d = decode_length_delimited(&mut buf).ok_or("bad not_in")?;
                    rules.not_in.push(d.to_vec());
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl RepeatedRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = RepeatedRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 0) => {
                    rules.min_items = Some(decode_varint(&mut buf).ok_or("bad min_items")?);
                }
                (2, 0) => {
                    rules.max_items = Some(decode_varint(&mut buf).ok_or("bad max_items")?);
                }
                (3, 0) => {
                    rules.unique = decode_varint(&mut buf).ok_or("bad unique")? != 0;
                }
                (4, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad items")?;
                    rules.items = Some(Box::new(FieldRules::decode_from_slice(sub)?));
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl MapRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = MapRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 0) => {
                    rules.min_pairs = Some(decode_varint(&mut buf).ok_or("bad min_pairs")?);
                }
                (2, 0) => {
                    rules.max_pairs = Some(decode_varint(&mut buf).ok_or("bad max_pairs")?);
                }
                (4, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad keys")?;
                    rules.keys = Some(Box::new(FieldRules::decode_from_slice(sub)?));
                }
                (5, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad values")?;
                    rules.values = Some(Box::new(FieldRules::decode_from_slice(sub)?));
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

// Numeric rules decoders for each wire format

impl NumericRules<f32> {
    pub fn decode_float(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = NumericRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 5) => {
                    if buf.len() < 4 {
                        return Err("short f32");
                    }
                    rules.r#const = Some(f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]));
                    buf = &buf[4..];
                }
                (2, 5) => {
                    if buf.len() < 4 {
                        return Err("short f32");
                    }
                    rules.lt = Some(f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]));
                    buf = &buf[4..];
                }
                (3, 5) => {
                    if buf.len() < 4 {
                        return Err("short f32");
                    }
                    rules.lte = Some(f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]));
                    buf = &buf[4..];
                }
                (4, 5) => {
                    if buf.len() < 4 {
                        return Err("short f32");
                    }
                    rules.gt = Some(f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]));
                    buf = &buf[4..];
                }
                (5, 5) => {
                    if buf.len() < 4 {
                        return Err("short f32");
                    }
                    rules.gte = Some(f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]));
                    buf = &buf[4..];
                }
                (6, 5) => {
                    if buf.len() < 4 {
                        return Err("short f32");
                    }
                    rules
                        .r#in
                        .push(f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]));
                    buf = &buf[4..];
                }
                (7, 5) => {
                    if buf.len() < 4 {
                        return Err("short f32");
                    }
                    rules
                        .not_in
                        .push(f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]));
                    buf = &buf[4..];
                }
                (8, 0) => {
                    rules.finite = decode_varint(&mut buf).ok_or("bad finite")? != 0;
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl NumericRules<f64> {
    pub fn decode_double(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = NumericRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 1) => {
                    if buf.len() < 8 {
                        return Err("short f64");
                    }
                    rules.r#const = Some(f64::from_le_bytes(buf[..8].try_into().unwrap()));
                    buf = &buf[8..];
                }
                (2, 1) => {
                    if buf.len() < 8 {
                        return Err("short f64");
                    }
                    rules.lt = Some(f64::from_le_bytes(buf[..8].try_into().unwrap()));
                    buf = &buf[8..];
                }
                (3, 1) => {
                    if buf.len() < 8 {
                        return Err("short f64");
                    }
                    rules.lte = Some(f64::from_le_bytes(buf[..8].try_into().unwrap()));
                    buf = &buf[8..];
                }
                (4, 1) => {
                    if buf.len() < 8 {
                        return Err("short f64");
                    }
                    rules.gt = Some(f64::from_le_bytes(buf[..8].try_into().unwrap()));
                    buf = &buf[8..];
                }
                (5, 1) => {
                    if buf.len() < 8 {
                        return Err("short f64");
                    }
                    rules.gte = Some(f64::from_le_bytes(buf[..8].try_into().unwrap()));
                    buf = &buf[8..];
                }
                (6, 1) => {
                    if buf.len() < 8 {
                        return Err("short f64");
                    }
                    rules
                        .r#in
                        .push(f64::from_le_bytes(buf[..8].try_into().unwrap()));
                    buf = &buf[8..];
                }
                (7, 1) => {
                    if buf.len() < 8 {
                        return Err("short f64");
                    }
                    rules
                        .not_in
                        .push(f64::from_le_bytes(buf[..8].try_into().unwrap()));
                    buf = &buf[8..];
                }
                (8, 0) => {
                    rules.finite = decode_varint(&mut buf).ok_or("bad finite")? != 0;
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

macro_rules! impl_varint_numeric_decoder {
    ($name:ident, $t:ty, $convert:expr) => {
        impl NumericRules<$t> {
            pub fn $name(data: &[u8]) -> Result<Self, &'static str> {
                let mut rules = NumericRules::default();
                let mut buf = data;
                let convert: fn(u64) -> $t = $convert;
                while !buf.is_empty() {
                    let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
                    match (field_number, wire_type) {
                        (1, 0) => {
                            rules.r#const =
                                Some(convert(decode_varint(&mut buf).ok_or("bad const")?));
                        }
                        (2, 0) => {
                            rules.lt = Some(convert(decode_varint(&mut buf).ok_or("bad lt")?));
                        }
                        (3, 0) => {
                            rules.lte = Some(convert(decode_varint(&mut buf).ok_or("bad lte")?));
                        }
                        (4, 0) => {
                            rules.gt = Some(convert(decode_varint(&mut buf).ok_or("bad gt")?));
                        }
                        (5, 0) => {
                            rules.gte = Some(convert(decode_varint(&mut buf).ok_or("bad gte")?));
                        }
                        (6, 0) => {
                            rules
                                .r#in
                                .push(convert(decode_varint(&mut buf).ok_or("bad in")?));
                        }
                        (6, 2) => {
                            let d = decode_length_delimited(&mut buf).ok_or("bad in packed")?;
                            let mut sub = d;
                            while !sub.is_empty() {
                                rules
                                    .r#in
                                    .push(convert(decode_varint(&mut sub).ok_or("bad in item")?));
                            }
                        }
                        (7, 0) => {
                            rules
                                .not_in
                                .push(convert(decode_varint(&mut buf).ok_or("bad not_in")?));
                        }
                        (7, 2) => {
                            let d = decode_length_delimited(&mut buf).ok_or("bad not_in packed")?;
                            let mut sub = d;
                            while !sub.is_empty() {
                                rules.not_in.push(convert(
                                    decode_varint(&mut sub).ok_or("bad not_in item")?,
                                ));
                            }
                        }
                        _ => {
                            skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                        }
                    }
                }
                Ok(rules)
            }
        }
    };
}

impl_varint_numeric_decoder!(
    decode_varint_signed32,
    i32,
    |v: u64| decode_i32_from_varint(v)
);
impl_varint_numeric_decoder!(
    decode_varint_signed64,
    i64,
    |v: u64| decode_i64_from_varint(v)
);
impl_varint_numeric_decoder!(decode_varint_u32, u32, |v: u64| v as u32);
impl_varint_numeric_decoder!(decode_varint_u64, u64, |v: u64| v);
impl_varint_numeric_decoder!(decode_sint32, i32, |v: u64| decode_sint32_from_varint(v));
impl_varint_numeric_decoder!(decode_sint64, i64, |v: u64| decode_sint64_from_varint(v));

macro_rules! impl_fixed_numeric_decoder {
    ($name:ident, $t:ty, $size:expr, $convert:expr) => {
        impl NumericRules<$t> {
            pub fn $name(data: &[u8]) -> Result<Self, &'static str> {
                let mut rules = NumericRules::default();
                let mut buf = data;
                let wire_type_expected: u8 = if $size == 4 { 5 } else { 1 };
                while !buf.is_empty() {
                    let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
                    if wire_type == wire_type_expected && (1..=7).contains(&field_number) {
                        if buf.len() < $size {
                            return Err("short fixed");
                        }
                        let bytes: [u8; $size] = buf[..$size].try_into().unwrap();
                        let val: $t = $convert(bytes);
                        *&mut buf = &buf[$size..];
                        match field_number {
                            1 => rules.r#const = Some(val),
                            2 => rules.lt = Some(val),
                            3 => rules.lte = Some(val),
                            4 => rules.gt = Some(val),
                            5 => rules.gte = Some(val),
                            6 => rules.r#in.push(val),
                            7 => rules.not_in.push(val),
                            _ => {}
                        }
                    } else {
                        skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                    }
                }
                Ok(rules)
            }
        }
    };
}

impl_fixed_numeric_decoder!(decode_fixed32, u32, 4, |b: [u8; 4]| u32::from_le_bytes(b));
impl_fixed_numeric_decoder!(decode_fixed64, u64, 8, |b: [u8; 8]| u64::from_le_bytes(b));
impl_fixed_numeric_decoder!(decode_sfixed32, i32, 4, |b: [u8; 4]| i32::from_le_bytes(b));
impl_fixed_numeric_decoder!(decode_sfixed64, i64, 8, |b: [u8; 8]| i64::from_le_bytes(b));

impl ProtoTimeval {
    fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut seconds: i64 = 0;
        let mut nanos: i32 = 0;
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (1, 0) => {
                    seconds = decode_i64_from_varint(decode_varint(&mut buf).ok_or("bad seconds")?)
                }
                (2, 0) => {
                    nanos = decode_i32_from_varint(decode_varint(&mut buf).ok_or("bad nanos")?)
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(ProtoTimeval { seconds, nanos })
    }
}

impl AnyRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = AnyRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (2, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad in")?;
                    rules.r#in.push(decode_string(s));
                }
                (3, 2) => {
                    let s = decode_length_delimited(&mut buf).ok_or("bad not_in")?;
                    rules.not_in.push(decode_string(s));
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl DurationRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = DurationRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (2, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad const")?;
                    rules.r#const = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                (3, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad lt")?;
                    rules.lt = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                (4, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad lte")?;
                    rules.lte = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                (5, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad gt")?;
                    rules.gt = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                (6, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad gte")?;
                    rules.gte = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                (7, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad in")?;
                    rules.r#in.push(ProtoTimeval::decode_from_slice(sub)?);
                }
                (8, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad not_in")?;
                    rules.not_in.push(ProtoTimeval::decode_from_slice(sub)?);
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}

impl TimestampRules {
    pub fn decode_from_slice(data: &[u8]) -> Result<Self, &'static str> {
        let mut rules = TimestampRules::default();
        let mut buf = data;
        while !buf.is_empty() {
            let (field_number, wire_type) = decode_tag(&mut buf).ok_or("bad tag")?;
            match (field_number, wire_type) {
                (2, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad const")?;
                    rules.r#const = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                (3, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad lt")?;
                    rules.lt = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                (4, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad lte")?;
                    rules.lte = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                (5, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad gt")?;
                    rules.gt = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                (6, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad gte")?;
                    rules.gte = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                (7, 0) => {
                    rules.lt_now = decode_varint(&mut buf).ok_or("bad lt_now")? != 0;
                }
                (8, 0) => {
                    rules.gt_now = decode_varint(&mut buf).ok_or("bad gt_now")? != 0;
                }
                (9, 2) => {
                    let sub = decode_length_delimited(&mut buf).ok_or("bad within")?;
                    rules.within = Some(ProtoTimeval::decode_from_slice(sub)?);
                }
                _ => {
                    skip_field(&mut buf, wire_type).ok_or("skip fail")?;
                }
            }
        }
        Ok(rules)
    }
}
