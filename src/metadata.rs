use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Error;
use cargo_metadata::{CargoOpt, Metadata, MetadataCommand};
use wapm_toml::Bindings;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MetadataTable {
    pub wapm: Wapm,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Wapm {
    pub namespace: String,
    pub package: Option<String>,
    pub wasmer_extra_flags: Option<String>,
    pub abi: wapm_toml::Abi,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fs: Option<HashMap<String, PathBuf>>,
    pub bindings: Option<Bindings>,
}

#[tracing::instrument(skip_all)]
pub(crate) fn parse_cargo_toml(
    manifest_path: Option<&Path>,
    no_default_features: bool,
    features: Option<&Features>,
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
        cmd.features(CargoOpt::SomeFeatures(features.0.clone()));
    }

    if all_features {
        cmd.features(CargoOpt::AllFeatures);
    }

    tracing::debug!(cmd = ?cmd.cargo_command(), "Parsing Cargo metadata");

    let meta = cmd.exec()?;

    Ok(meta)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Features(pub Vec<String>);

impl From<&'_ str> for Features {
    fn from(value: &'_ str) -> Self {
        Features(value.split(',').map(|s| s.to_string()).collect())
    }
}
