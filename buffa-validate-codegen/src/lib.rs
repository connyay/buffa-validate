pub mod constraints;
mod field;
pub mod generated;
mod message;
mod rules;
#[cfg(feature = "connectrpc")]
mod service;

use anyhow::Result;
use buffa_codegen::generated::descriptor::FileDescriptorProto;
use buffa_codegen::{GeneratedFile, GeneratedFileKind, context::CodeGenContext};

pub fn generate_validation(
    file_descriptors: &[FileDescriptorProto],
    files_to_generate: &[String],
    codegen_config: &buffa_codegen::CodeGenConfig,
) -> Result<Vec<GeneratedFile>> {
    let ctx = CodeGenContext::for_generate(file_descriptors, files_to_generate, codegen_config);
    let mut companions = Vec::new();

    for file_name in files_to_generate {
        let file = file_descriptors
            .iter()
            .find(|f| f.name.as_deref() == Some(file_name))
            .ok_or_else(|| anyhow::anyhow!("file not found: {file_name}"))?;

        let package = file.package.clone().unwrap_or_default();

        let mut tokens = message::generate_file_validations(file, &package, &ctx)?;

        #[cfg(feature = "connectrpc")]
        {
            let service_tokens = service::generate_file_services(file, &ctx)?;
            tokens.extend(service_tokens);
        }

        if tokens.is_empty() {
            continue;
        }

        let code = format_token_stream(&tokens)?;
        let stem = buffa_codegen::proto_path_to_stem(file_name);
        companions.push(GeneratedFile {
            name: format!("{stem}.__validate.rs"),
            package,
            kind: GeneratedFileKind::Companion,
            content: code,
        });
    }

    Ok(companions)
}

fn format_token_stream(tokens: &proc_macro2::TokenStream) -> Result<String> {
    let file = syn::parse2(tokens.clone())
        .map_err(|e| anyhow::anyhow!("generated code does not parse: {e}"))?;
    Ok(prettyplease::unparse(&file))
}
