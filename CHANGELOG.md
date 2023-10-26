# Changelog

## [0.4.0](https://github.com/wasmerio/cargo-wasmer/compare/cargo-wasmer-v0.3.6...cargo-wasmer-v0.4.0) (2023-10-26)

### ⚠ BREAKING CHANGES

- Renamed the `cargo wapm` command to `cargo wasmer`

### Features

- renamed `cargo wapm` command to `cargo wasmer` ([ca2934d](https://github.com/wasmerio/cargo-wasmer/commit/ca2934d5d18f71851e701c888a84aa1dc1327cd2))

### Bug Fixes

* Added the cargo-wapm executable back in for backwards compatibility ([0cfa867](https://github.com/wasmerio/cargo-wasmer/commit/0cfa867f978e71cce791e3dd4c11f9b66953cba8))
* Bumped the proc-macro2 version because it used nightly features that no longer exist ([a0982e8](https://github.com/wasmerio/cargo-wasmer/commit/a0982e8773f091b249131712f99f95a126ec9fa7))
* For backwards compatibility, made sure the publishing will still respect the `[package.metadata.wapm]` table ([099cc36](https://github.com/wasmerio/cargo-wasmer/commit/099cc368ffcf1c1ccd55726460e47152ea26802b))
