use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub package: Package,
    #[serde(rename = "module", default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<Module>,
    #[serde(rename = "command", default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<Command>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub fs: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub readme: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub wasmer_extra_flags: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Module {
    pub name: String,
    pub source: String,
    pub abi: Abi,
    pub bindings: Option<Bindings>,
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Abi {
    Wasi,
    Emscripten,
    None,
}

impl Abi {
    pub fn target(self) -> &'static str {
        match self {
            Abi::Wasi => "wasm32-wasi",
            Abi::Emscripten => "wasm32-unknown-emscripten",
            Abi::None => "wasm32-unknown-unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Command {
    pub name: String,
    pub module: String,
    pub package: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Bindings {
    /// The `*.wit` file's location on disk.
    #[serde(alias = "wit-exports")]
    pub exports: PathBuf,
    /// The version of the WIT format being used.
    pub wit_bindgen: String,
}
