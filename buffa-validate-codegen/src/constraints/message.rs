use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_nested_validation(
    field_ident: &Ident,
    field_path: &str,
    is_message_field: bool,
) -> TokenStream {
    if is_message_field {
        quote! {
            if let ::core::option::Option::Some(__nested) = self.#field_ident.as_option()
                && let ::core::result::Result::Err(nested_violations) = ::buffa_validate::Validate::validate(__nested)
            {
                for mut v in nested_violations.violations {
                    if v.field_path.is_empty() {
                        v.field_path = ::std::string::String::from(#field_path);
                    } else {
                        v.field_path = ::std::format!("{}.{}", #field_path, v.field_path);
                    }
                    violations.push(v);
                }
            }
        }
    } else {
        quote! {
            if let ::core::result::Result::Err(nested_violations) = ::buffa_validate::Validate::validate(&self.#field_ident) {
                for mut v in nested_violations.violations {
                    if v.field_path.is_empty() {
                        v.field_path = ::std::string::String::from(#field_path);
                    } else {
                        v.field_path = ::std::format!("{}.{}", #field_path, v.field_path);
                    }
                    violations.push(v);
                }
            }
        }
    }
}
