use buffa_codegen::generated::descriptor::{
    DescriptorProto, FieldDescriptorProto, OneofDescriptorProto,
};

use crate::generated::{self, FieldRules, MessageRules, OneofRules};

pub fn field_rules(field: &FieldDescriptorProto) -> Option<FieldRules> {
    if !field.options.is_set() {
        return None;
    }
    generated::extract_field_rules(&field.options.__buffa_unknown_fields)
}

pub fn message_rules(message: &DescriptorProto) -> Option<MessageRules> {
    if !message.options.is_set() {
        return None;
    }
    generated::extract_message_rules(&message.options.__buffa_unknown_fields)
}

pub fn oneof_rules(oneof: &OneofDescriptorProto) -> Option<OneofRules> {
    if !oneof.options.is_set() {
        return None;
    }
    generated::extract_oneof_rules(&oneof.options.__buffa_unknown_fields)
}
