use anyhow::Error;
use cargo_wasmer::Publish;
use clap::Parser;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "warn,cargo_wasmer=info");
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let args = Cargo::parse();
    tracing::debug!(?args, "Started");

    match args {
        Cargo::Wasmer(p) => p.execute(),
    }
}

#[derive(Debug, Parser)]
#[clap(name = "cargo", bin_name = "cargo", version, author)]
enum Cargo {
    #[clap(alias = "wapm")]
    Wasmer(Publish),
}
