use anyhow::Result;
use buffa_codegen::context::CodeGenContext;
use buffa_codegen::generated::descriptor::FieldDescriptorProto;
use buffa_codegen::generated::descriptor::field_descriptor_proto::Type;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::constraints;
use crate::generated::{FieldRules, Ignore, TypeRules};

pub fn generate_field_validation(
    field_desc: &FieldDescriptorProto,
    rules: &FieldRules,
    field_name: &str,
    _message_fqn: &str,
    _ctx: &CodeGenContext<'_>,
) -> Result<TokenStream> {
    if rules.ignore == Ignore::Always {
        return Ok(TokenStream::new());
    }

    let field_ident = buffa_codegen::idents::make_field_ident(field_name);
    let field_path_str = field_name;

    let is_optional = field_desc.proto3_optional.unwrap_or(false);
    let is_message_type = field_desc.r#type == Some(Type::TYPE_MESSAGE);

    let mut checks = TokenStream::new();

    // Required check
    if rules.required
        && (is_optional || is_message_type) {
            let field_path = field_path_str.to_string();
            checks.extend(quote! {
                if self.#field_ident.is_none() {
                    violations.push(::buffa_validate::Violation::new(
                        #field_path,
                        "required",
                        "value is required",
                    ));
                }
            });
        }

    // Type-specific constraint checks
    if let Some(type_rules) = &rules.type_rules {
        let constraint_checks =
            generate_type_checks(type_rules, &field_ident, field_path_str, is_optional)?;
        checks.extend(constraint_checks);
    }

    Ok(checks)
}

fn generate_type_checks(
    type_rules: &TypeRules,
    field_ident: &Ident,
    field_path: &str,
    is_optional: bool,
) -> Result<TokenStream> {
    match type_rules {
        TypeRules::String(rules) => {
            let inner = constraints::string::generate(rules, field_path)?;
            if is_optional {
                Ok(quote! {
                    if let ::core::option::Option::Some(ref val) = self.#field_ident {
                        let __field_val: &str = val.as_ref();
                        #inner
                    }
                })
            } else {
                Ok(quote! {
                    {
                        let __field_val: &str = &self.#field_ident;
                        #inner
                    }
                })
            }
        }
        TypeRules::Int32(rules) => {
            let inner = constraints::numeric::generate_int(rules, field_path, "i32")?;
            wrap_optional(field_ident, is_optional, inner)
        }
        TypeRules::Int64(rules) => {
            let inner = constraints::numeric::generate_int(rules, field_path, "i64")?;
            wrap_optional(field_ident, is_optional, inner)
        }
        TypeRules::Uint32(rules) => {
            let inner = constraints::numeric::generate_uint(rules, field_path, "u32")?;
            wrap_optional(field_ident, is_optional, inner)
        }
        TypeRules::Uint64(rules) => {
            let inner = constraints::numeric::generate_uint(rules, field_path, "u64")?;
            wrap_optional(field_ident, is_optional, inner)
        }
        TypeRules::Float(rules) => {
            let inner = constraints::numeric::generate_float(rules, field_path)?;
            wrap_optional(field_ident, is_optional, inner)
        }
        TypeRules::Double(rules) => {
            let inner = constraints::numeric::generate_double(rules, field_path)?;
            wrap_optional(field_ident, is_optional, inner)
        }
        TypeRules::Bool(rules) => {
            let inner = constraints::bool::generate(rules, field_path)?;
            wrap_optional(field_ident, is_optional, inner)
        }
        TypeRules::Enum(rules) => {
            let inner = constraints::enumeration::generate(rules, field_path)?;
            wrap_optional(field_ident, is_optional, inner)
        }
        TypeRules::Repeated(rules) => {
            constraints::repeated::generate(rules, field_ident, field_path)
        }
        TypeRules::Map(rules) => constraints::map::generate(rules, field_ident, field_path),
        TypeRules::Bytes(rules) => {
            let inner = constraints::bytes::generate(rules, field_path)?;
            if is_optional {
                Ok(quote! {
                    if let ::core::option::Option::Some(ref val) = self.#field_ident {
                        let __field_val: &[u8] = val.as_ref();
                        #inner
                    }
                })
            } else {
                Ok(quote! {
                    {
                        let __field_val: &[u8] = &self.#field_ident;
                        #inner
                    }
                })
            }
        }
        _ => Ok(TokenStream::new()),
    }
}

fn wrap_optional(
    field_ident: &Ident,
    is_optional: bool,
    inner: TokenStream,
) -> Result<TokenStream> {
    if is_optional {
        Ok(quote! {
            if let ::core::option::Option::Some(ref __field_val) = self.#field_ident {
                let __field_val = *__field_val;
                #inner
            }
        })
    } else {
        Ok(quote! {
            {
                let __field_val = self.#field_ident;
                #inner
            }
        })
    }
}
