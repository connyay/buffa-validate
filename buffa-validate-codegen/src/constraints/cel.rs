use anyhow::Result;
use buffa_codegen::generated::descriptor::FieldDescriptorProto;
use buffa_codegen::generated::descriptor::field_descriptor_proto::Type;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::generated::{Rule, TypeRules};

pub fn generate_field_cel(
    rules: &[Rule],
    field_ident: &Ident,
    field_path: &str,
    type_rules: Option<&TypeRules>,
    is_optional: bool,
    field_type: Option<Type>,
) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    for (i, rule) in rules.iter().enumerate() {
        let expr = match &rule.expression {
            Some(e) if !e.is_empty() => e.as_str(),
            _ => continue,
        };
        let rule_id = rule.id.as_deref().unwrap_or("cel");
        let default_message = rule.message.as_deref().unwrap_or("CEL validation failed");

        let cel_static_name = quote::format_ident!(
            "__CEL_FIELD_{}_{}",
            field_path.replace('.', "_").to_uppercase(),
            i
        );
        let to_cel_value = this_conversion(type_rules, field_type);

        if is_optional {
            checks.extend(quote! {
                if let ::core::option::Option::Some(ref __cel_val) = self.#field_ident {
                    static #cel_static_name: ::std::sync::OnceLock<::buffa_validate::__private::cel::Program> =
                        ::std::sync::OnceLock::new();
                    let __prog = #cel_static_name.get_or_init(|| ::buffa_validate::helpers::cel_compile(#expr));
                    let mut __ctx = ::buffa_validate::__private::cel::Context::default();
                    let __this_val = #to_cel_value;
                    __ctx.add_variable_from_value("this", __this_val);
                    if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::cel_check(__prog, &__ctx, #rule_id, #default_message) {
                        violations.push(::buffa_validate::Violation::new(#field_path, #rule_id, msg));
                    }
                }
            });
        } else {
            checks.extend(quote! {
                {
                    static #cel_static_name: ::std::sync::OnceLock<::buffa_validate::__private::cel::Program> =
                        ::std::sync::OnceLock::new();
                    let __prog = #cel_static_name.get_or_init(|| ::buffa_validate::helpers::cel_compile(#expr));
                    let mut __ctx = ::buffa_validate::__private::cel::Context::default();
                    let __cel_val = &self.#field_ident;
                    let __this_val = #to_cel_value;
                    __ctx.add_variable_from_value("this", __this_val);
                    if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::cel_check(__prog, &__ctx, #rule_id, #default_message) {
                        violations.push(::buffa_validate::Violation::new(#field_path, #rule_id, msg));
                    }
                }
            });
        }
    }

    Ok(checks)
}

pub fn generate_message_cel(
    rules: &[Rule],
    fields: &[FieldDescriptorProto],
) -> Result<TokenStream> {
    let mut checks = TokenStream::new();

    let mut map_inserts = TokenStream::new();
    for field_desc in fields {
        let field_name = field_desc.name.as_deref().unwrap_or("");
        let field_ident = buffa_codegen::idents::make_field_ident(field_name);
        let is_optional = field_desc.proto3_optional.unwrap_or(false);
        let conversion = field_type_to_cel_value(field_desc.r#type, is_optional, &field_ident);
        map_inserts.extend(quote! {
            __map.insert(
                ::std::string::String::from(#field_name),
                #conversion,
            );
        });
    }

    for (i, rule) in rules.iter().enumerate() {
        let expr = match &rule.expression {
            Some(e) if !e.is_empty() => e.as_str(),
            _ => continue,
        };
        let rule_id = rule.id.as_deref().unwrap_or("cel");
        let default_message = rule.message.as_deref().unwrap_or("CEL validation failed");

        let cel_static_name = quote::format_ident!("__CEL_MSG_{}", i);

        checks.extend(quote! {
            {
                static #cel_static_name: ::std::sync::OnceLock<::buffa_validate::__private::cel::Program> =
                    ::std::sync::OnceLock::new();
                let __prog = #cel_static_name.get_or_init(|| ::buffa_validate::helpers::cel_compile(#expr));
                let mut __ctx = ::buffa_validate::__private::cel::Context::default();
                let mut __map = ::std::collections::HashMap::<::std::string::String, ::buffa_validate::__private::cel::Value>::new();
                #map_inserts
                let __this_val: ::buffa_validate::__private::cel::Value = __map.into();
                __ctx.add_variable_from_value("this", __this_val);
                if let ::core::option::Option::Some(msg) = ::buffa_validate::helpers::cel_check(__prog, &__ctx, #rule_id, #default_message) {
                    violations.push(::buffa_validate::Violation::new("", #rule_id, msg));
                }
            }
        });
    }

    Ok(checks)
}

fn field_type_to_cel_value(
    field_type: Option<Type>,
    is_optional: bool,
    field_ident: &Ident,
) -> TokenStream {
    if is_optional {
        let inner = scalar_to_cel_value(field_type);
        return quote! {
            match self.#field_ident {
                ::core::option::Option::Some(ref __v) => #inner,
                ::core::option::Option::None => ::buffa_validate::__private::cel::Value::Null,
            }
        };
    }

    match field_type {
        Some(Type::TYPE_STRING) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_string(&self.#field_ident) }
        }
        Some(Type::TYPE_BYTES) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_bytes(&self.#field_ident) }
        }
        Some(Type::TYPE_BOOL) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_bool(self.#field_ident) }
        }
        Some(Type::TYPE_INT32 | Type::TYPE_SINT32 | Type::TYPE_SFIXED32) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(self.#field_ident as i64) }
        }
        Some(Type::TYPE_INT64 | Type::TYPE_SINT64 | Type::TYPE_SFIXED64) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(self.#field_ident) }
        }
        Some(Type::TYPE_UINT32 | Type::TYPE_FIXED32) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_uint(self.#field_ident as u64) }
        }
        Some(Type::TYPE_UINT64 | Type::TYPE_FIXED64) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_uint(self.#field_ident) }
        }
        Some(Type::TYPE_FLOAT) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_float(self.#field_ident as f64) }
        }
        Some(Type::TYPE_DOUBLE) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_float(self.#field_ident) }
        }
        Some(Type::TYPE_ENUM) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(self.#field_ident.to_i32() as i64) }
        }
        _ => {
            quote! { ::buffa_validate::__private::cel::Value::Null }
        }
    }
}

fn scalar_to_cel_value(field_type: Option<Type>) -> TokenStream {
    match field_type {
        Some(Type::TYPE_STRING) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_string(__v) }
        }
        Some(Type::TYPE_BYTES) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_bytes(__v) }
        }
        Some(Type::TYPE_BOOL) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_bool(*__v) }
        }
        Some(Type::TYPE_INT32 | Type::TYPE_SINT32 | Type::TYPE_SFIXED32) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(*__v as i64) }
        }
        Some(Type::TYPE_INT64 | Type::TYPE_SINT64 | Type::TYPE_SFIXED64) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(*__v) }
        }
        Some(Type::TYPE_UINT32 | Type::TYPE_FIXED32) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_uint(*__v as u64) }
        }
        Some(Type::TYPE_UINT64 | Type::TYPE_FIXED64) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_uint(*__v) }
        }
        Some(Type::TYPE_FLOAT) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_float(*__v as f64) }
        }
        Some(Type::TYPE_DOUBLE) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_float(*__v) }
        }
        Some(Type::TYPE_ENUM) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(__v.to_i32() as i64) }
        }
        _ => {
            quote! { ::buffa_validate::__private::cel::Value::Null }
        }
    }
}

fn this_conversion(type_rules: Option<&TypeRules>, field_type: Option<Type>) -> TokenStream {
    match type_rules {
        Some(TypeRules::String(_)) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_string(__cel_val) }
        }
        Some(TypeRules::Bytes(_)) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_bytes(__cel_val) }
        }
        Some(TypeRules::Bool(_)) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_bool(*__cel_val) }
        }
        Some(TypeRules::Int32(_) | TypeRules::Sint32(_) | TypeRules::Sfixed32(_)) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(*__cel_val as i64) }
        }
        Some(TypeRules::Int64(_) | TypeRules::Sint64(_) | TypeRules::Sfixed64(_)) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(*__cel_val) }
        }
        Some(TypeRules::Uint32(_) | TypeRules::Fixed32(_)) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_uint(*__cel_val as u64) }
        }
        Some(TypeRules::Uint64(_) | TypeRules::Fixed64(_)) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_uint(*__cel_val) }
        }
        Some(TypeRules::Float(_)) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_float(*__cel_val as f64) }
        }
        Some(TypeRules::Double(_)) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_float(*__cel_val) }
        }
        Some(TypeRules::Enum(_)) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(__cel_val.to_i32() as i64) }
        }
        None => this_conversion_from_field_type(field_type),
        _ => {
            quote! { ::buffa_validate::__private::cel::Value::Null }
        }
    }
}

fn this_conversion_from_field_type(field_type: Option<Type>) -> TokenStream {
    match field_type {
        Some(Type::TYPE_STRING) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_string(__cel_val) }
        }
        Some(Type::TYPE_BYTES) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_bytes(__cel_val) }
        }
        Some(Type::TYPE_BOOL) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_bool(*__cel_val) }
        }
        Some(Type::TYPE_INT32 | Type::TYPE_SINT32 | Type::TYPE_SFIXED32) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(*__cel_val as i64) }
        }
        Some(Type::TYPE_INT64 | Type::TYPE_SINT64 | Type::TYPE_SFIXED64) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(*__cel_val) }
        }
        Some(Type::TYPE_UINT32 | Type::TYPE_FIXED32) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_uint(*__cel_val as u64) }
        }
        Some(Type::TYPE_UINT64 | Type::TYPE_FIXED64) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_uint(*__cel_val) }
        }
        Some(Type::TYPE_FLOAT) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_float(*__cel_val as f64) }
        }
        Some(Type::TYPE_DOUBLE) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_float(*__cel_val) }
        }
        Some(Type::TYPE_ENUM) => {
            quote! { ::buffa_validate::helpers::field_to_cel_value_int(__cel_val.to_i32() as i64) }
        }
        _ => {
            quote! { ::buffa_validate::__private::cel::Value::Null }
        }
    }
}
