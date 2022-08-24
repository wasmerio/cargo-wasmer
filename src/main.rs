mod manifest;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Error};
use cargo_metadata::{CargoOpt, Metadata, MetadataCommand, Package, Target};
use clap::Parser;
use serde::Deserialize;
use tracing_subscriber::EnvFilter;

use crate::manifest::{Abi, Bindings, Manifest, Module};

fn main() -> Result<(), Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "warn,cargo_wapm=info");
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let mut all_args: Vec<_> = std::env::args().collect();

    if std::env::var("CARGO").is_ok() {
        tracing::debug!("Note: running as a cargo subcommand");
        all_args.remove(0);
    }

    let args = Args::parse_from(all_args);

    tracing::debug!(?args, "Started");

    run(&args)?;

    Ok(())
}

/// Publish a crate to the WebAssembly Package Manager.
#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, env)]
    dry_run: bool,
    #[clap(long, env)]
    manifest_path: Option<PathBuf>,
    #[clap(short, long, env)]
    workspace: bool,
    /// A comma-delimited list of features to enable.
    #[clap(long)]
    features: Option<String>,
    #[clap(long)]
    all_features: bool,
    #[clap(long)]
    no_default_features: bool,
    /// Packages to ignore.
    #[clap(long)]
    exclude: Vec<String>,
    /// Compile in debug mode.
    #[clap(long)]
    debug: bool,
}

fn run(args: &Args) -> Result<(), Error> {
    let Args {
        manifest_path,
        workspace,
        no_default_features,
        all_features,
        features,
        exclude,
        ..
    } = args;
    let features: Option<Vec<_>> = features
        .as_ref()
        .map(|f| f.split(",").map(ToString::to_string).collect());

    let metadata = parse_metadata(
        manifest_path.as_deref(),
        *no_default_features,
        features,
        *all_features,
    )
    .context("Unable to parse the workspace's metadata")?;

    let current_dir =
        std::env::current_dir().context("Unable to determine the current directory")?;

    let packages_to_publish =
        determine_crates_to_publish(&metadata, *workspace, &current_dir, &exclude)
            .context("Unable to determine which crates to publish")?;

    let dir = metadata.target_directory.join("wapm");

    tracing::debug!(%dir, "Clearing the output directory");

    for pkg in packages_to_publish {
        let dest: PathBuf = dir.join(&pkg.name).into();
        publish(pkg, metadata.target_directory.as_ref(), &dest, args)
            .with_context(|| format!("Unable to publish \"{}\"", pkg.name))?;
    }

    Ok(())
}

#[tracing::instrument(fields(pkg = pkg.name.as_str()), skip_all)]
fn publish(pkg: &Package, target_dir: &Path, dir: &Path, args: &Args) -> Result<(), Error> {
    tracing::info!(dry_run = args.dry_run, "Publishing");

    let target = determine_target(pkg)?;
    let manifest: Manifest = generate_manifest(&pkg, target)?;
    let wasm_path = compile_to_wasm(pkg, target_dir, args.debug, &manifest.modules[0], target)?;
    pack(dir, &manifest, &wasm_path, pkg)?;
    upload_to_wapm(&dir, args.dry_run)?;

    tracing::info!("Published!");

    Ok(())
}

fn determine_target(pkg: &Package) -> Result<&Target, Error> {
    let candidates: Vec<_> = pkg
        .targets
        .iter()
        .filter(|t| is_webassembly_library(t) || is_binary(t))
        .collect();
    match *candidates.as_slice() {
        [single_target] => Ok(single_target),
        [] => anyhow::bail!(
            "The {} package doesn't contain any binaries or \"cdylib\" libraries",
            pkg.name
        ),
        [..] => anyhow::bail!(
            "Unable to decide what to publish. Expected one executable or \"cdylib\" library, but found {}",
            candidates.iter()
                .map(|t| format!("{} ({:?})", t.name, t.kind))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[tracing::instrument(skip_all)]
fn upload_to_wapm(dir: &Path, dry_run: bool) -> Result<(), Error> {
    let mut cmd = Command::new("wapm");

    cmd.arg("publish");
    if dry_run {
        cmd.arg("--dry-run");
    }

    cmd.current_dir(dir);

    tracing::debug!(?cmd, "Publishing with the wapm CLI");

    let status = cmd.status().with_context(|| {
        format!(
            "Unable to start \"{}\". Is it installed?",
            cmd.get_program().to_string_lossy()
        )
    })?;

    if !status.success() {
        match status.code() {
            Some(code) => {
                anyhow::bail!("The wapm CLI exited unsuccessfully with exit code {}", code)
            }
            None => anyhow::bail!("The wapm CLI exited unsuccessfully"),
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
fn pack(dir: &Path, manifest: &Manifest, wasm_path: &Path, pkg: &Package) -> Result<(), Error> {
    std::fs::create_dir_all(dir)
        .with_context(|| format!("Unable to create the \"{}\" directory", dir.display()))?;

    let manifest_path = dir.join("wapm.toml");
    let toml = toml::to_string(manifest).context("Unable to serialize the wapm.toml")?;
    tracing::debug!(
        path = %manifest_path.display(),
        bytes = toml.len(),
        "Writing manifest",
    );
    std::fs::write(&manifest_path, toml.as_bytes())
        .with_context(|| format!("Unable to write to \"{}\"", manifest_path.display()))?;

    let new_wasm_path = dir.join(wasm_path.file_name().unwrap());
    copy(wasm_path, new_wasm_path)?;

    let base_dir = pkg.manifest_path.parent().unwrap();

    if let Some(license_file) = pkg.license_file.as_ref() {
        let license_file = base_dir.join(&license_file);
        let dest = dir.join(Path::new(&license_file).file_name().unwrap());
        copy(license_file, dest)?;
    }

    if let Some(readme) = pkg.readme.as_ref() {
        let readme = base_dir.join(&readme);
        let dest = dir.join(readme.file_name().unwrap());
        copy(readme, dest)?;
    }

    for module in &manifest.modules {
        if let Some(Bindings { wit_exports, .. }) = &module.bindings {
            // TODO: Recursively check for any *.wit files this might pull in
            let bindings = base_dir.as_std_path().join(wit_exports);
            let dest = dir.join(wit_exports.file_name().unwrap());
            copy(bindings, dest)?;
        }
    }

    Ok(())
}

fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), Error> {
    let from = from.as_ref();
    let to = to.as_ref();

    tracing::debug!(
        from = %from.display(),
        to = %to.display(),
        "Copying file",
    );
    std::fs::copy(from, to).with_context(|| {
        format!(
            "Unable to copy \"{}\" to \"{}\"",
            from.display(),
            to.display()
        )
    })?;

    Ok(())
}

#[tracing::instrument(skip_all)]
fn compile_to_wasm(
    pkg: &Package,
    target_dir: &Path,
    debug: bool,
    module: &Module,
    target: &Target,
) -> Result<PathBuf, Error> {
    let mut cmd = Command::new(cargo_bin());
    let target_triple = module.abi.target();

    cmd.arg("build")
        .arg("--quiet")
        .args(&["--manifest-path", pkg.manifest_path.as_str()])
        .args(&["--target", target_triple]);

    if !debug {
        cmd.arg("--release");
    }

    tracing::debug!(?cmd, "Compiling the WebAssembly package");

    let status = cmd.status().with_context(|| {
        format!(
            "Unable to start \"{}\". Is it installed?",
            cmd.get_program().to_string_lossy()
        )
    })?;

    if !status.success() {
        match status.code() {
            Some(code) => anyhow::bail!("Cargo exited unsuccessfully with exit code {}", code),
            None => anyhow::bail!("Cargo exited unsuccessfully"),
        }
    }

    let binary = target_dir
        .join(target_triple)
        .join(if debug { "debug" } else { "release" })
        .join(wasm_binary_name(target))
        .with_extension("wasm");

    anyhow::ensure!(
        binary.exists(),
        "Expected \"{}\" to exist",
        binary.display()
    );

    Ok(binary)
}

fn wasm_binary_name(target: &Target) -> String {
    // Because reasons, `rustc` will leave dashes in a binary's name but
    // libraries are converted to underscores.
    if is_binary(target) {
        target.name.clone()
    } else {
        target.name.replace('-', "_")
    }
}

fn cargo_bin() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"))
}

fn is_webassembly_library(target: &Target) -> bool {
    target.kind.iter().any(|k| k == "cdylib")
}

fn is_binary(target: &Target) -> bool {
    target.kind.iter().any(|k| k == "bin")
}

#[tracing::instrument(skip_all)]
fn generate_manifest(pkg: &Package, target: &Target) -> Result<Manifest, Error> {
    tracing::trace!(?target, "The target");

    let MetadataTable {
        wapm:
            Wapm {
                wasmer_extra_flags,
                fs,
                abi,
                namespace,
                package,
                bindings,
            },
    } = MetadataTable::deserialize(&pkg.metadata)
        .context("Unable to deserialize the [metadata] table")?;

    match pkg.description.as_deref() {
        Some("") => anyhow::bail!("The \"description\" field in your Cargo.toml is empty"),
        Some(_) => {}
        None => anyhow::bail!("The \"description\" field in your Cargo.toml wasn't set"),
    }

    let package_name = format!("{}/{}", namespace, package.as_deref().unwrap_or(&pkg.name));

    let module = Module {
        name: target.name.clone(),
        source: format!("{}.wasm", wasm_binary_name(target)),
        abi,
        bindings,
    };

    let mut commands = Vec::new();
    if is_binary(target) {
        commands.push(manifest::Command {
            module: target.name.clone(),
            name: target.name.clone(),
            package: package_name.clone(),
        });
    }

    Ok(Manifest {
        package: crate::manifest::Package {
            name: package_name,
            version: pkg.version.to_string(),
            description: pkg.description.clone().unwrap_or_default(),
            license: pkg.license.clone(),
            license_file: pkg
                .license_file()
                .and_then(|p| p.file_name().map(String::from)),
            readme: pkg.readme().and_then(|p| p.file_name().map(String::from)),
            repository: pkg.repository.clone(),
            homepage: pkg.homepage.clone(),
            wasmer_extra_flags,
        },
        modules: vec![module],
        commands,
        fs,
    })
}

#[tracing::instrument(skip_all)]
fn determine_crates_to_publish<'meta>(
    metadata: &'meta Metadata,
    workspace: bool,
    current_dir: &Path,
    exclude: &[String],
) -> Result<Vec<&'meta Package>, Error> {
    tracing::debug!("Determining which crates to publish");

    let all_workspace_members: Vec<_> = metadata
        .packages
        .iter()
        .filter(|pkg| metadata.workspace_members.contains(&pkg.id))
        .collect();

    if workspace {
        tracing::debug!("Looking for publishable packages in the workspace");
        let mut packages = Vec::new();

        for pkg in all_workspace_members {
            let _span =
                tracing::debug_span!("Checking package", name = pkg.name.as_str()).entered();

            if exclude.contains(&pkg.name) {
                tracing::debug!("Explicitly ignoring");
                continue;
            }

            if pkg
                .metadata
                .as_object()
                .and_then(|m| m.get("wapm"))
                .is_none()
            {
                tracing::debug!(
                    "Skipping because it doesn't contain a [package.metadata.wapm] table"
                );
                continue;
            }

            packages.push(pkg);
        }

        Ok(packages)
    } else {
        // We want to find which package to publish based on the user's current
        // directory, however it's possible that you can have nested packages
        // so we want to get the most specific one.
        let mut candidates: Vec<_> = all_workspace_members
            .into_iter()
            .filter(|pkg| {
                let dir = pkg.manifest_path.parent().unwrap();
                current_dir.starts_with(dir)
            })
            .collect();
        candidates.sort_by_key(|pkg| pkg.manifest_path.components().count());

        if let Some(&pkg) = candidates.last() {
            Ok(vec![pkg])
        } else if let Some(root) = metadata.root_package() {
            // use the "root" package as a default.
            Ok(vec![root])
        } else {
            anyhow::bail!("Unable to determine which package to publish. Either \"cd\" into the crate folder or use the \"--workspace\" flag.");
        }
    }
}

#[tracing::instrument(skip_all)]
fn parse_metadata(
    manifest_path: Option<&Path>,
    no_default_features: bool,
    features: Option<Vec<String>>,
    all_features: bool,
) -> Result<Metadata, Error> {
    let mut cmd = MetadataCommand::new();

    if let Some(manifest_path) = manifest_path {
        cmd.manifest_path(manifest_path);
    }

    if let Ok(path) = std::env::current_dir() {
        cmd.current_dir(path);
    }

    if no_default_features {
        cmd.features(CargoOpt::NoDefaultFeatures);
    }

    if let Some(features) = features {
        cmd.features(CargoOpt::SomeFeatures(features));
    }

    if all_features {
        cmd.features(CargoOpt::AllFeatures);
    }

    tracing::debug!(cmd = ?cmd.cargo_command(), "Parsing Cargo metadata");

    let meta = cmd.exec()?;

    Ok(meta)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MetadataTable {
    wapm: Wapm,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Wapm {
    namespace: String,
    package: Option<String>,
    wasmer_extra_flags: Option<String>,
    abi: Abi,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    fs: HashMap<String, String>,
    bindings: Option<Bindings>,
}
