use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use buffa::Message;
use buffa_codegen::generated::descriptor::FileDescriptorSet;

#[derive(Debug, Clone, Default)]
enum DescriptorSource {
    #[default]
    Protoc,
    Buf,
    Precompiled(PathBuf),
}

pub struct Config {
    files: Vec<PathBuf>,
    includes: Vec<PathBuf>,
    out_dir: Option<PathBuf>,
    descriptor_source: DescriptorSource,
    include_file: Option<String>,
    emit_rerun_directives: bool,
    codegen_config: buffa_codegen::CodeGenConfig,
    #[cfg(feature = "connectrpc")]
    connect_options: connectrpc_codegen::codegen::Options,
}

impl Config {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            includes: Vec::new(),
            out_dir: None,
            descriptor_source: DescriptorSource::default(),
            include_file: None,
            emit_rerun_directives: true,
            codegen_config: buffa_codegen::CodeGenConfig::default(),
            #[cfg(feature = "connectrpc")]
            connect_options: connectrpc_codegen::codegen::Options::default(),
        }
    }

    #[must_use]
    pub fn files(mut self, files: &[impl AsRef<Path>]) -> Self {
        self.files
            .extend(files.iter().map(|f| f.as_ref().to_path_buf()));
        self
    }

    #[must_use]
    pub fn includes(mut self, includes: &[impl AsRef<Path>]) -> Self {
        self.includes
            .extend(includes.iter().map(|i| i.as_ref().to_path_buf()));
        self
    }

    #[must_use]
    pub fn out_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.out_dir = Some(dir.into());
        self
    }

    #[must_use]
    pub fn use_buf(mut self) -> Self {
        self.descriptor_source = DescriptorSource::Buf;
        self
    }

    #[must_use]
    pub fn descriptor_set(mut self, path: impl Into<PathBuf>) -> Self {
        self.descriptor_source = DescriptorSource::Precompiled(path.into());
        self
    }

    #[must_use]
    pub fn include_file(mut self, name: impl Into<String>) -> Self {
        self.include_file = Some(name.into());
        self
    }

    #[must_use]
    pub fn emit_rerun_directives(mut self, enabled: bool) -> Self {
        self.emit_rerun_directives = enabled;
        self
    }

    #[must_use]
    pub fn generate_json(mut self, enabled: bool) -> Self {
        self.codegen_config.generate_json = enabled;
        #[cfg(feature = "connectrpc")]
        {
            self.connect_options.buffa.generate_json = enabled;
        }
        self
    }

    #[must_use]
    pub fn file_per_package(mut self, enabled: bool) -> Self {
        self.codegen_config.file_per_package = enabled;
        #[cfg(feature = "connectrpc")]
        {
            self.connect_options.buffa.file_per_package = enabled;
        }
        self
    }

    pub fn compile(self) -> Result<()> {
        let relative_includes = self.out_dir.is_some();
        let out_dir = match self.out_dir {
            Some(d) => d,
            None => std::env::var_os("OUT_DIR")
                .map(PathBuf::from)
                .context("OUT_DIR is not set and no out_dir() was configured")?,
        };

        // 1. Acquire descriptors
        let (descriptor_bytes, files_to_generate) = match &self.descriptor_source {
            DescriptorSource::Protoc => {
                let bytes = run_protoc(&self.files, &self.includes)?;
                let mut includes = self.includes.clone();
                includes.sort_by_key(|p| std::cmp::Reverse(p.as_os_str().len()));
                let files = self
                    .files
                    .iter()
                    .map(|f| strip_include_prefix(f, &includes))
                    .filter(|s| !s.is_empty())
                    .collect();
                (bytes, files)
            }
            DescriptorSource::Buf => {
                let bytes = run_buf(&self.files)?;
                (bytes, proto_relative_names(&self.files))
            }
            DescriptorSource::Precompiled(p) => {
                let bytes = std::fs::read(p)
                    .with_context(|| format!("failed to read descriptor set '{}'", p.display()))?;
                (bytes, proto_relative_names(&self.files))
            }
        };
        let fds = FileDescriptorSet::decode_from_slice(&descriptor_bytes)
            .map_err(|e| anyhow!("failed to decode FileDescriptorSet: {e}"))?;

        // 2. Generate message types (+ service code if connectrpc feature)
        #[cfg(feature = "connectrpc")]
        let mut generated = connectrpc_codegen::codegen::generate_files(
            &fds.file,
            &files_to_generate,
            &self.connect_options,
        )?;

        #[cfg(not(feature = "connectrpc"))]
        let mut generated =
            buffa_codegen::generate(&fds.file, &files_to_generate, &self.codegen_config)
                .map_err(|e| anyhow!("buffa-codegen failed: {e}"))?;

        // 3. Generate validation companions
        let codegen_config_for_validate = {
            #[cfg(feature = "connectrpc")]
            {
                self.connect_options.buffa.clone()
            }
            #[cfg(not(feature = "connectrpc"))]
            {
                self.codegen_config.clone()
            }
        };
        let validation_companions = buffa_validate_codegen::generate_validation(
            &fds.file,
            &files_to_generate,
            &codegen_config_for_validate,
        )?;

        // 4. Merge validation companions into stitchers
        if !validation_companions.is_empty() {
            buffa_codegen::apply_companions(&mut generated, validation_companions);
        }

        // 5. Write all files
        std::fs::create_dir_all(&out_dir)
            .with_context(|| format!("failed to create out_dir '{}'", out_dir.display()))?;

        let mut entries: Vec<(String, String)> = Vec::new();
        for file in &generated {
            let path = out_dir.join(&file.name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            write_if_changed(&path, file.content.as_bytes())?;
            if file.kind == buffa_codegen::GeneratedFileKind::PackageMod {
                entries.push((file.name.clone(), file.package.clone()));
            }
        }

        // 6. Optional include file
        if let Some(ref include_name) = self.include_file {
            let include_src = generate_include_file(&entries, relative_includes);
            let include_path = out_dir.join(include_name);
            write_if_changed(&include_path, include_src.as_bytes())?;
        }

        // 7. Cargo rerun directives
        if self.emit_rerun_directives {
            match &self.descriptor_source {
                DescriptorSource::Precompiled(p) => {
                    println!("cargo:rerun-if-changed={}", p.display());
                }
                DescriptorSource::Buf => {}
                DescriptorSource::Protoc => {
                    for f in &self.files {
                        println!("cargo:rerun-if-changed={}", f.display());
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

fn write_if_changed(path: &Path, content: &[u8]) -> std::io::Result<()> {
    if let Ok(existing) = std::fs::read(path)
        && existing == content
    {
        return Ok(());
    }
    std::fs::write(path, content)
}

fn run_protoc(files: &[PathBuf], includes: &[PathBuf]) -> Result<Vec<u8>> {
    let protoc = std::env::var("PROTOC").unwrap_or_else(|_| "protoc".to_string());

    let out = tempfile::NamedTempFile::new().context("failed to create tempfile for protoc")?;
    let out_path = out.path().to_path_buf();

    let mut cmd = Command::new(&protoc);
    cmd.arg("--include_imports");
    cmd.arg(format!("--descriptor_set_out={}", out_path.display()));
    for inc in includes {
        cmd.arg(format!("--proto_path={}", inc.display()));
    }
    for f in files {
        cmd.arg(f.as_os_str());
    }

    let output = cmd
        .output()
        .with_context(|| format!("failed to spawn protoc ('{protoc}')"))?;
    if !output.status.success() {
        bail!("protoc failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    std::fs::read(&out_path).context("failed to read protoc descriptor output")
}

fn run_buf(files: &[PathBuf]) -> Result<Vec<u8>> {
    let out = tempfile::NamedTempFile::new().context("failed to create tempfile for buf")?;
    let out_path = out.path().to_path_buf();

    let mut cmd = Command::new("buf");
    cmd.arg("build")
        .arg("--as-file-descriptor-set")
        .arg("-o")
        .arg(&out_path);
    for f in files {
        cmd.arg("--path").arg(f.as_os_str());
    }

    let output = cmd.output().context("failed to spawn buf")?;
    if !output.status.success() {
        bail!(
            "buf build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    std::fs::read(&out_path).context("failed to read buf descriptor output")
}

fn strip_include_prefix(f: &Path, includes: &[PathBuf]) -> String {
    for inc in includes {
        if let Ok(stripped) = f.strip_prefix(inc) {
            let s = stripped.to_string_lossy().replace('\\', "/");
            if !s.is_empty() {
                return s;
            }
        }
    }
    f.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn proto_relative_names(files: &[PathBuf]) -> Vec<String> {
    files
        .iter()
        .map(|f| f.to_string_lossy().replace('\\', "/"))
        .collect()
}

fn generate_include_file(entries: &[(String, String)], relative: bool) -> String {
    let include_mode = if relative {
        buffa_codegen::IncludeMode::Relative("")
    } else {
        buffa_codegen::IncludeMode::OutDir
    };
    buffa_codegen::generate_module_tree(entries, include_mode, false)
}
