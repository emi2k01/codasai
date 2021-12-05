use std::path::PathBuf;

use anyhow::{Context, Result};
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Opts {
    /// Title of the guide
    title: String,
    /// Path to the directory that will contain the project
    #[structopt(short, long, default_value = "./")]
    path: PathBuf,
}

pub fn execute(opts: &Opts) -> Result<()> {
    std::fs::create_dir_all(&opts.path)
        .with_context(|| format!("failed to create directory {:?}", &opts.path))?;

    let path = opts
        .path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize path {:?}", &opts.path))?;

    git2::Repository::init(&path)
        .with_context(|| format!("failed to initialize git repository in {:?} ", &path))?;

    let dotcodasai = path.join(".codasai");
    std::fs::create_dir_all(&dotcodasai)
        .with_context(|| format!("failed to create .codasai directory in {:?}", &dotcodasai))?;

    let guide_toml = dotcodasai.join("guide.toml");
    let title = escape_toml_string(&opts.title);
    std::fs::write(&guide_toml, format!("title = \"{}\"", title))?;

    Ok(())
}

fn escape_toml_string(s: &str) -> String {
    let mut escaped = String::new();
    for ch in s.chars() {
        match ch {
            ch if ch >= 0 as char && ch <= 0x1f as char => escaped.extend(ch.escape_default()),
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            ch => escaped.push(ch),
        }
    }
    escaped
}
