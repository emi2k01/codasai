mod code;
mod commands;
mod exporter;
mod html;
mod page;
mod paths;

use anyhow::Result;
use commands::{build, init, preview};
use env_logger::Env;
use structopt::StructOpt;

#[derive(StructOpt)]
enum Args {
    /// Initializes a codasai project
    Init(init::Opts),
    /// Preview the current unsaved page
    Preview(preview::Opts),
    /// Build the guide
    Build(build::Opts),
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::from_args();

    match args {
        Args::Init(ref opts) => init::execute(opts),
        Args::Preview(ref opts) => preview::execute(opts),
        Args::Build(ref opts) => build::execute(opts),
    }
}
