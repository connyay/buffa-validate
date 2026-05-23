use anyhow::Result;
use buffa_codegen::context::CodeGenContext;
use buffa_codegen::generated::descriptor::{FileDescriptorProto, ServiceDescriptorProto};
use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::rules;

pub fn generate_file_services(
    file: &FileDescriptorProto,
    ctx: &CodeGenContext<'_>,
) -> Result<TokenStream> {
    let mut tokens = TokenStream::new();
    let package = file.package.as_deref().unwrap_or("");

    for service in &file.service {
        let service_tokens = generate_validated_service(file, service, package, ctx)?;
        tokens.extend(service_tokens);
    }

    Ok(tokens)
}

fn generate_validated_service(
    file: &FileDescriptorProto,
    service: &ServiceDescriptorProto,
    package: &str,
    ctx: &CodeGenContext<'_>,
) -> Result<TokenStream> {
    let service_name = service.name.as_deref().unwrap_or("");
    let service_upper = service_name.to_upper_camel_case();
    let trait_name = format_ident!("{}", service_upper);
    let validated_name = format_ident!("Validated{}", service_upper);
    let generic_s = format_ident!("S");

    let mut has_validatable_methods = false;
    let mut method_impls = Vec::new();

    for method in &service.method {
        let method_name = method.name.as_deref().unwrap_or("");
        let method_snake = buffa_codegen::idents::make_field_ident(&method_name.to_snake_case());
        let client_streaming = method.client_streaming.unwrap_or(false);
        let server_streaming = method.server_streaming.unwrap_or(false);

        let input_fqn = method.input_type.as_deref().unwrap_or("");
        let output_fqn = method.output_type.as_deref().unwrap_or("");

        let input_view = resolve_owned_view_type(input_fqn, package, ctx)?;
        let output_type_tokens = resolve_message_type(output_fqn, package, ctx)?;

        let input_has_validation = has_validation_rules(file, input_fqn);

        if client_streaming && server_streaming {
            method_impls.push(quote! {
                async fn #method_snake(
                    &self,
                    ctx: ::connectrpc::RequestContext,
                    requests: ::connectrpc::ServiceStream<#input_view>,
                ) -> ::connectrpc::ServiceResult<::connectrpc::ServiceStream<impl ::connectrpc::Encodable<#output_type_tokens> + Send + use<#generic_s>>> {
                    self.0.#method_snake(ctx, requests).await
                }
            });
        } else if client_streaming {
            method_impls.push(quote! {
                async fn #method_snake<'a>(
                    &'a self,
                    ctx: ::connectrpc::RequestContext,
                    requests: ::connectrpc::ServiceStream<#input_view>,
                ) -> ::connectrpc::ServiceResult<impl ::connectrpc::Encodable<#output_type_tokens> + Send + use<'a, #generic_s>> {
                    self.0.#method_snake(ctx, requests).await
                }
            });
        } else if server_streaming {
            if input_has_validation {
                has_validatable_methods = true;
                method_impls.push(quote! {
                    async fn #method_snake(
                        &self,
                        ctx: ::connectrpc::RequestContext,
                        request: #input_view,
                    ) -> ::connectrpc::ServiceResult<::connectrpc::ServiceStream<impl ::connectrpc::Encodable<#output_type_tokens> + Send + use<#generic_s>>> {
                        ::buffa_validate::Validate::validate(&*request)
                            .map_err(::buffa_validate::violations_to_connect_error)?;
                        self.0.#method_snake(ctx, request).await
                    }
                });
            } else {
                method_impls.push(quote! {
                    async fn #method_snake(
                        &self,
                        ctx: ::connectrpc::RequestContext,
                        request: #input_view,
                    ) -> ::connectrpc::ServiceResult<::connectrpc::ServiceStream<impl ::connectrpc::Encodable<#output_type_tokens> + Send + use<#generic_s>>> {
                        self.0.#method_snake(ctx, request).await
                    }
                });
            }
        } else {
            // Unary
            if input_has_validation {
                has_validatable_methods = true;
                method_impls.push(quote! {
                    async fn #method_snake<'a>(
                        &'a self,
                        ctx: ::connectrpc::RequestContext,
                        request: #input_view,
                    ) -> ::connectrpc::ServiceResult<impl ::connectrpc::Encodable<#output_type_tokens> + Send + use<'a, #generic_s>> {
                        ::buffa_validate::Validate::validate(&*request)
                            .map_err(::buffa_validate::violations_to_connect_error)?;
                        self.0.#method_snake(ctx, request).await
                    }
                });
            } else {
                method_impls.push(quote! {
                    async fn #method_snake<'a>(
                        &'a self,
                        ctx: ::connectrpc::RequestContext,
                        request: #input_view,
                    ) -> ::connectrpc::ServiceResult<impl ::connectrpc::Encodable<#output_type_tokens> + Send + use<'a, #generic_s>> {
                        self.0.#method_snake(ctx, request).await
                    }
                });
            }
        }
    }

    if !has_validatable_methods {
        return Ok(TokenStream::new());
    }

    Ok(quote! {
        pub struct #validated_name<#generic_s>(pub #generic_s);

        impl<#generic_s: #trait_name> #trait_name for #validated_name<#generic_s> {
            #(#method_impls)*
        }
    })
}

fn resolve_message_type(
    proto_fqn: &str,
    _package: &str,
    ctx: &CodeGenContext<'_>,
) -> Result<TokenStream> {
    let rust_path = ctx
        .rust_type(proto_fqn)
        .ok_or_else(|| anyhow::anyhow!("no rust type for {proto_fqn}"))?;
    Ok(buffa_codegen::idents::rust_path_to_tokens(rust_path))
}

fn resolve_owned_view_type(
    proto_fqn: &str,
    package: &str,
    ctx: &CodeGenContext<'_>,
) -> Result<TokenStream> {
    let split = ctx
        .rust_type_relative_split(proto_fqn, package, 0)
        .ok_or_else(|| anyhow::anyhow!("no rust type for {proto_fqn}"))?;
    let prefix = if split.to_package.is_empty() {
        format!("{}::view", buffa_codegen::context::SENTINEL_MOD)
    } else {
        format!(
            "{}::{}::view",
            split.to_package,
            buffa_codegen::context::SENTINEL_MOD
        )
    };
    let view_path = format!("{prefix}::{}View", split.within_package);
    let view_tokens = buffa_codegen::idents::rust_path_to_tokens(&view_path);
    Ok(quote! { ::buffa::view::OwnedView<#view_tokens<'static>> })
}

fn has_validation_rules(file: &FileDescriptorProto, proto_fqn: &str) -> bool {
    let type_name = proto_fqn
        .strip_prefix('.')
        .unwrap_or(proto_fqn)
        .rsplit('.')
        .next()
        .unwrap_or(proto_fqn);

    find_message_has_rules(file, type_name)
}

fn find_message_has_rules(file: &FileDescriptorProto, name: &str) -> bool {
    for message in &file.message_type {
        if message.name.as_deref() == Some(name) {
            if rules::message_rules(message).is_some() {
                return true;
            }
            for field in &message.field {
                if rules::field_rules(field).is_some() {
                    return true;
                }
            }
            for oneof in &message.oneof_decl {
                if rules::oneof_rules(oneof).is_some() {
                    return true;
                }
            }
        }
    }
    false
}
