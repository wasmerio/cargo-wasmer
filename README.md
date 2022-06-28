# Cargo WAPM

[![Continuous integration](https://github.com/hotg-ai/cargo-wapm/workflows/Continuous%20integration/badge.svg?branch=main)](https://github.com/hotg-ai/cargo-wapm/actions)

([API Docs])

A `cargo` sub-command for publishing Rust crates to the WebAssembly Package
Manager.

## Getting Started

You can install the `cargo wapm` command from crates.io.

```console
$ cargo install cargo-wapm
```

You will also need to [install the `wapm` CLI][install-wapm] and
[authenticate][auth] with WAPM.

```console
$ curl https://get.wasmer.io -sSfL | sh
$ wapm login
Username: my-user
Password: ****
```

Once you have done that, open the `Cargo.toml` for your crate and add a metadata
section to tell `cargo wapm` how your crate will be packaged.

```toml
# Cargo.toml
[package.metadata.wapm]
namespace = "Michael-F-Bryan"
abi = "none"
```

The `abi` argument tells `cargo wapm` which target to use when compiling to
WebAssembly.

| ABI          | Target Triple            |
| ------------ | ------------------------ |
| `none`       | `wasm32-unknown-unknown` |
| `wasi`       | `wasm32-wasi`            |
| `emscripten` | `wasm32-emscripten`      |

You also need to add `cdylib` to the `crate-type` list. You should also add the
`rlib` crate type if other crates depend on this crate (integration tests, doc
tests, examples, etc.).

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]
```

Now the `Cargo.toml` is up to date, we can do a dry run to make sure everything
is correct.

```console
$ cd examples/hello-world/
$ cargo wapm --dry-run
2022-05-03T17:33:31.929353Z  INFO publish: cargo_wapm: Publishing dry_run=true pkg="hello-world"
Successfully published package `Michael-F-Bryan/hello-world@0.1.0`
[INFO] Publish succeeded, but package was not published because it was run in dry-run mode
2022-05-03T17:33:32.366576Z  INFO publish: cargo_wapm: Published! pkg="hello-world"
```

We can see that some files have been written to the `target/wapm/` folder.

```console
$ tree ../../target/wapm
../../target/wapm
└── hello-world
    ├── hello_world.wasm
    ├── LICENSE_MIT.md
    ├── README.md
    └── wapm.toml

1 directory, 4 files

$ cat ../../target/wapm/hello-world/wapm.toml
[package]
name = "Michael-F-Bryan/hello-world"
version = "0.1.0"
description = "A dummy package."
license-file = "LICENSE_MIT.md"
readme = "README.md"

[[module]]
name = "hello-world"
source = "hello_world.wasm"
abi = "none"
```

If you are happy with the generated files, remove the `--dry-run` command to
publish the crate for real.

The `cargo wapm` command doesn't take care of any version bumping, so the
`version` being published is read directly from `Cargo.toml`. Check out [the
`cargo release` tool](https://crates.io/crates/cargo-release) if you something
that can manage routine release tasks like bumping versions, tagging commits, or
updating your changelog.

## Workspaces

Normally, the `cargo wapm` command will only publish the crate in the current
directory.

However, by using the `--workspace` flag you can publish every crate in the
current workspace as long as they have a `[package.metadata.wapm]` section in
their `Cargo.toml`. The `--exclude` argument lets you skip a particular crate
while publishing.

## License

This project is licensed under the Apache License, Version 2.0
([LICENSE-APACHE](LICENSE.md) or http://www.apache.org/licenses/LICENSE-2.0).

It is recommended to always use [cargo-crev][crev] to verify the
trustworthiness of each of your dependencies, including this one.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

The intent of this crate is to be free of soundness bugs. The developers will
do their best to avoid them, and welcome help in analysing and fixing them.

[API Docs]: https://hotg-ai.github.io/cargo-wapm
[crev]: https://github.com/crev-dev/cargo-crev
[install-wapm]: https://docs.wasmer.io/ecosystem/wapm/getting-started
[wapm-auth]: https://docs.wasmer.io/ecosystem/wapm/publishing-your-package#creating-an-account-in-wapm
