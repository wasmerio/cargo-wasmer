use anyhow::Error;
use cargo_wapm::Wapm;
use clap::Parser;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "warn,cargo_wapm=info");
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let args = Cargo::parse();
    tracing::debug!(?args, "Started");

    match args {
        Cargo::Wapm(Wapm::Publish(p)) => p.execute(),
    }
}

#[derive(Debug, Parser)]
#[clap(name = "cargo", bin_name = "cargo", version, author)]
enum Cargo {
    #[clap(subcommand)]
    Wapm(Wapm),
}
