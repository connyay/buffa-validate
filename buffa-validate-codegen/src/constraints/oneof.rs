use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_required(oneof_ident: &Ident, field_path: &str) -> TokenStream {
    quote! {
        if self.#oneof_ident.is_none() {
            violations.push(::buffa_validate::Violation::new(
                #field_path,
                "required",
                "exactly one field must be set in oneof",
            ));
        }
    }
}
