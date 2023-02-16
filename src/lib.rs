pub extern crate clap_cargo;

pub mod metadata;
mod pack;
mod publish;

pub use crate::{pack::Pack, publish::Publish};

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, about, version)]
pub enum Wapm {
    Publish(Publish),
}
