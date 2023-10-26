# Cargo Wasmer

[![Continuous Integration](https://github.com/wasmerio/cargo-wasmer/actions/workflows/ci.yml/badge.svg)](https://github.com/wasmerio/cargo-wasmer/actions/workflows/ci.yml)

([API Docs])

A `cargo` sub-command for publishing Rust crates to the [Wasmer package
registry](https://wasmer.io/).

If you want a deeper understanding of how `cargo wasmer` works, check out
[*Announcing Cargo WAPM*][announcement].

> **Note:** This command used to be called `cargo wapm` back when `wapm` was a
> separate command. Most interactions with the Wasmer registry have been merged
> into the `wasmer` CLI nowadays, so the command was renamed to `cargo wasmer`.

## Getting Started

You can install the `cargo wasmer` command from crates.io.

```console
$ cargo install cargo-wasmer --locked
```

You will also need to [install the `wasmer` CLI][install-wasmer] and
[authenticate][auth] with the registry.

```console
$ curl https://get.wasmer.io -sSfL | sh
$ wasmer login
Username: my-user
Password: ****
```

Once you have done that, open the `Cargo.toml` for your crate and add a metadata
section to tell `cargo wasmer` how your crate will be packaged.

```toml
# Cargo.toml
[package.metadata.wasmer]
namespace = "Michael-F-Bryan"
abi = "none"
```

The `abi` argument tells `cargo wasmer` which target to use when compiling to
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
$ cargo wasmer --dry-run
2022-05-03T17:33:31.929353Z  INFO publish: cargo_wasmer: Publishing dry_run=true pkg="hello-world"
Successfully published package `Michael-F-Bryan/hello-world@0.1.0`
[INFO] Publish succeeded, but package was not published because it was run in dry-run mode
2022-05-03T17:33:32.366576Z  INFO publish: cargo_wasmer: Published! pkg="hello-world"
```

We can see that some files have been written to the `target/wasmer/` folder.

```console
$ tree ../../target/wasmer
../../target/wasmer
└── hello-world
    ├── hello_world.wasm
    ├── LICENSE_MIT.md
    ├── README.md
    └── wasmer.toml

1 directory, 4 files

$ cat ../../target/wasmer/hello-world/wasmer.toml
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

The `cargo wasmer` command doesn't take care of any version bumping, so the
`version` being published is read directly from `Cargo.toml`. Check out [the
`cargo release` tool](https://crates.io/crates/cargo-release) if you something
that can manage routine release tasks like bumping versions, tagging commits, or
updating your changelog.

## Workspaces

Normally, the `cargo wasmer` command will only publish the crate in the current
directory.

However, by using the `--workspace` flag you can publish every crate in the
current workspace as long as they have a `[package.metadata.wasmer]` section in
their `Cargo.toml`. The `--exclude` argument lets you skip a particular crate
while publishing.

### Releasing

This repository uses [Release Please][release-please] to automate a lot of the
work around creating releases.

Every time a commit following the [Conventional Commit Style][conv] is merged
into `main`, the [`release-please.yml`](.github/workflows/release-please.yml)
workflow will run and update the "Release PR" to reflect the new changes.

For commits that just fix bugs (i.e. the message starts with `"fix: "`), the
associated crate will receive a changelog entry and a patch version bump.
Similarly, adding a new feature (i.e. `"feat:"`) does a minor version bump and
adding breaking changes (i.e. `"fix!:"` or `"feat!:"`) will result in a major
version bump.

When the release PR is merged, the updated changelogs and bumped version numbers
will be merged into the `main` branch, the `release-please.yml` workflow will
automatically generate GitHub Releases, and CI will publish the crate if
necessary.

TL;DR:

1. Use [Conventional Commit Messages][conv] whenever you make a noteworthy change
2. Merge the release PR when ready to release
3. Let the automation do everything else

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

[announcement]: https://adventures.michaelfbryan.com/posts/announcing-cargo-wapm/
[API Docs]: https://wasmerio.github.io/cargo-wasmer
[auth]: https://docs.wasmer.io/registry/get-started#log-in-into-wasmer
[conv]: https://www.conventionalcommits.org/en/v1.0.0/
[crev]: https://github.com/crev-dev/cargo-crev
[install-wasmer]: https://docs.wasmer.io/install
[release-please]: https://github.com/googleapis/release-please
