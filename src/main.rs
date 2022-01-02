mod code;
mod commands;
mod context;
mod export;
mod html;
mod page;
mod paths;

use anyhow::Result;
use clap::Parser;
use commands::{build, init, preview, save};
use env_logger::Env;

#[derive(Parser)]
enum Args {
    /// Initializes a codasai project.
    Init(init::Opts),
    /// Preview the current unsaved page.
    Preview(preview::Opts),
    /// Build the guide.
    Build(build::Opts),
    /// Saves the newly added page.
    ///
    /// This makes a git commit that includes the new page, your workspace directory and anything that
    /// you already have staged.
    Save(save::Opts),
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    match args {
        Args::Init(ref opts) => init::execute(opts),
        Args::Preview(ref opts) => preview::execute(opts),
        Args::Build(ref opts) => build::execute(opts),
        Args::Save(ref opts) => save::execute(opts),
    }
}
