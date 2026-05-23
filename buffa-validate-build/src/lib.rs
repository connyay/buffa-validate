use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use buffa::Message;
use buffa_codegen::generated::descriptor::FileDescriptorSet;

const VALIDATE_PROTO: &[u8] = include_bytes!("../../proto/buf/validate/validate.proto");

/// Writes the bundled `buf/validate/validate.proto` into `OUT_DIR` and returns
/// the include path that resolves `import "buf/validate/validate.proto"`.
///
/// Pass this to `.includes()` on any protoc-based build crate
/// (`connectrpc_build`, `buffa_build`, etc.) so consumers don't need to vendor
/// the proto themselves.
pub fn include_dir() -> PathBuf {
    let out_dir: PathBuf = std::env::var_os("OUT_DIR")
        .expect("OUT_DIR not set — must be called from a build script")
        .into();
    let root = out_dir.join("buffa-validate-protos");
    let proto_path = root.join("buf/validate/validate.proto");
    if !proto_path.exists() {
        std::fs::create_dir_all(proto_path.parent().unwrap()).unwrap();
        std::fs::write(&proto_path, VALIDATE_PROTO).unwrap();
    }
    root
}

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
    emit_rerun_directives: bool,
    codegen_config: buffa_codegen::CodeGenConfig,
}

impl Config {
    pub fn new() -> Self {
        let mut codegen_config = buffa_codegen::CodeGenConfig::default();
        codegen_config.generate_views = true;
        Self {
            files: Vec::new(),
            includes: Vec::new(),
            out_dir: None,
            descriptor_source: DescriptorSource::default(),
            emit_rerun_directives: true,
            codegen_config,
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
    pub fn emit_rerun_directives(mut self, enabled: bool) -> Self {
        self.emit_rerun_directives = enabled;
        self
    }

    /// Override the buffa CodeGenConfig used for type resolution.
    ///
    /// Only needed when the upstream build uses non-default settings that
    /// affect type paths (e.g. `extern_path`). Must match the config used
    /// by the upstream buffa or connectrpc build.
    #[must_use]
    pub fn buffa_config(mut self, config: buffa_codegen::CodeGenConfig) -> Self {
        self.codegen_config = config;
        self
    }

    pub fn compile(self) -> Result<()> {
        let out_dir = match self.out_dir {
            Some(d) => d,
            None => std::env::var_os("OUT_DIR")
                .map(PathBuf::from)
                .context("OUT_DIR is not set and no out_dir() was configured")?,
        };

        // 1. Acquire descriptors
        let (descriptor_bytes, files_to_generate) = match &self.descriptor_source {
            DescriptorSource::Protoc => {
                let mut includes = self.includes.clone();
                includes.push(include_dir());
                let bytes = run_protoc(&self.files, &includes)?;
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

        // 2. Generate validation companions
        let companions = buffa_validate_codegen::generate_validation(
            &fds.file,
            &files_to_generate,
            &self.codegen_config,
        )?;

        if companions.is_empty() {
            return Ok(());
        }

        // 3. Write companion files and patch existing stitcher files
        for comp in &companions {
            let comp_path = out_dir.join(&comp.name);
            write_if_changed(&comp_path, comp.content.as_bytes())?;
            patch_stitcher(&out_dir, &comp.package, &comp.name)?;
        }

        // 4. Cargo rerun directives
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

fn patch_stitcher(out_dir: &Path, package: &str, companion_name: &str) -> Result<()> {
    let stitcher_path = find_stitcher(out_dir, package)?;
    let content = std::fs::read_to_string(&stitcher_path)
        .with_context(|| format!("failed to read stitcher '{}'", stitcher_path.display()))?;

    let include_line = format!("include!(\"{companion_name}\");");
    if content.contains(&include_line) {
        return Ok(());
    }

    let patched = format!("{content}{include_line}\n");
    std::fs::write(&stitcher_path, patched.as_bytes())
        .with_context(|| format!("failed to patch stitcher '{}'", stitcher_path.display()))
}

fn find_stitcher(out_dir: &Path, package: &str) -> Result<PathBuf> {
    let mod_path = out_dir.join(format!("{package}.mod.rs"));
    if mod_path.exists() {
        return Ok(mod_path);
    }
    let pkg_path = out_dir.join(format!("{package}.rs"));
    if pkg_path.exists() {
        return Ok(pkg_path);
    }
    bail!(
        "no stitcher file found for package '{package}' in {}; \
         ensure the upstream buffa or connectrpc build runs first",
        out_dir.display()
    )
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
