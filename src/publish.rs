use std::{path::Path, process::Command};

use anyhow::{Context, Error};
use cargo_metadata::Package;
use clap::Parser;

use crate::Pack;

/// Publish a crate to the WebAssembly Package Manager.
#[derive(Debug, Parser)]
#[clap(author, about, version)]
pub struct Publish {
    /// Build the package, but don't publish it.
    #[clap(short, long, env)]
    pub dry_run: bool,
    #[clap(flatten)]
    pub pack: Pack,
}

impl Publish {
    /// Run the [`Publish`] command.
    pub fn execute(self) -> Result<(), Error> {
        let metadata = self
            .pack
            .metadata()
            .context("Unable to parse the workspace's metadata")?;

        let packages_to_publish = self.pack.resolve_packages(&metadata);

        for pkg in packages_to_publish {
            // We only want to publish things that have a
            // [package.metadata.wasmer] table
            if !has_package_metadata_table(pkg, "wasmer")
                && !has_package_metadata_table(pkg, "wapm")
            {
                tracing::info!(
                    pkg.name = pkg.name,
                    "No [package.metadata.wasmer] found in the package. Skipping..."
                );
                continue;
            }

            self.publish(pkg, metadata.target_directory.as_ref())
                .with_context(|| format!("Unable to publish \"{}\"", pkg.name))?;
        }

        Ok(())
    }

    #[tracing::instrument(fields(pkg = pkg.name.as_str()), skip_all)]
    fn publish(&self, pkg: &Package, target_dir: &Path) -> Result<(), Error> {
        tracing::info!(dry_run = self.dry_run, "Getting ready to publish");

        let dest = self.pack.generate_wasmer_package(pkg, target_dir)?;
        upload_to_wasmer(&dest, self.dry_run)?;

        tracing::info!("Published!");

        Ok(())
    }
}

fn has_package_metadata_table(pkg: &Package, table_name: &str) -> bool {
    pkg.metadata
        .as_object()
        .map(|obj| obj.contains_key(table_name))
        .unwrap_or(false)
}

#[tracing::instrument(skip_all)]
fn upload_to_wasmer(dir: &Path, dry_run: bool) -> Result<(), Error> {
    let mut cmd = Command::new("wasmer");

    cmd.arg("publish");
    if dry_run {
        cmd.arg("--dry-run");
    }

    cmd.current_dir(dir);

    tracing::debug!(?cmd, "Publishing with the wasmer CLI");

    let status = cmd.status().with_context(|| {
        format!(
            "Unable to start \"{}\". Is it installed?",
            cmd.get_program().to_string_lossy()
        )
    })?;

    if !status.success() {
        match status.code() {
            Some(code) => {
                anyhow::bail!(
                    "The wasmer CLI exited unsuccessfully with exit code {}",
                    code
                )
            }
            None => anyhow::bail!("The wasmer CLI exited unsuccessfully"),
        }
    }

    Ok(())
}
