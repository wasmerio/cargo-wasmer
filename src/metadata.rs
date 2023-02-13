use std::{collections::HashMap, path::PathBuf};

use wapm_toml::Bindings;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MetadataTable {
    pub wapm: Wapm,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use wapm_toml::{WaiBindings, WitBindings};

    use super::*;

    #[test]
    fn parse_wai_bindings() {
        let table = toml::toml! {
            [wapm]
            namespace = "wasmer"
            abi = "none"
            bindings = { wai-version = "0.1.0", exports = "hello-world.wai" }
        };
        let should_be = MetadataTable {
            wapm: Wapm {
                namespace: "wasmer".to_string(),
                package: None,
                wasmer_extra_flags: None,
                abi: wapm_toml::Abi::None,
                fs: None,
                bindings: Some(Bindings::Wai(WaiBindings {
                    exports: Some("hello-world.wai".into()),
                    imports: Vec::new(),
                    wai_version: "0.1.0".parse().unwrap(),
                })),
            },
        };

        let got = MetadataTable::deserialize(table).unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_wit_bindings() {
        let table = toml::toml! {
            [wapm]
            namespace = "wasmer"
            abi = "none"
            bindings = { wit-bindgen = "0.1.0", wit-exports = "hello-world.wit" }
        };
        let should_be = MetadataTable {
            wapm: Wapm {
                namespace: "wasmer".to_string(),
                package: None,
                wasmer_extra_flags: None,
                abi: wapm_toml::Abi::None,
                fs: None,
                bindings: Some(Bindings::Wit(WitBindings {
                    wit_bindgen: "0.1.0".parse().unwrap(),
                    wit_exports: "hello-world.wit".into(),
                })),
            },
        };

        let got = MetadataTable::deserialize(table).unwrap();

        assert_eq!(got, should_be);
    }
}
