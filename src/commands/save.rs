use std::process::Command;

use anyhow::{Context, Result};
use structopt::StructOpt;

use crate::exporter::{Index, IndexEntry};

#[derive(StructOpt)]
pub struct Opts {}

pub fn execute(_opts: &Opts) -> Result<()> {
    let project = crate::paths::project()
        .context("current directory is not part of a Codasai project")?
        .canonicalize()
        .context("failed to canonicalize project directory")?;

    let mut index = Index::from_project(&project)?;

    let new_page_path = crate::page::find_new_page(&project).context("failed to find new page")?;
    let new_page_path = new_page_path.ok_or(anyhow::anyhow!("there are no unsaved pages"))?;
    let new_page_content = std::fs::read_to_string(&new_page_path)
        .with_context(|| format!("failed to read new page at {:?}", &new_page_path))?;
    let new_page_title = crate::page::extract_title(&new_page_content);
    let new_page_file_name = new_page_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
        .strip_suffix(".md")
        .expect("pages should have `.md` extension")
        .to_owned();

    index.entries.push(IndexEntry {
        name: new_page_title.clone(),
        code: new_page_file_name.clone(),
    });

    index.write_to_project(&project)?;

    let git_add_output = Command::new("git")
        .args(&["add", "-A"])
        .output()
        .context("failed to invoke \"git add -A\"")?;

    if !git_add_output.status.success() {
        let git_error = String::from_utf8_lossy(&git_add_output.stderr);
        anyhow::bail!("`git add` exited with error:\n\n{}", git_error);
    }

    let git_commit_message = format!(
        "Add page: {}\nCode: {}",
        &new_page_title, &new_page_file_name
    );
    let git_commit_output = Command::new("git")
        .args(&[
            "-c",
            "committer.name=Codasai CLI",
            "-c",
            "committer.email=codasai.cli@gmail.com",
            "commit",
            "-m",
            &git_commit_message,
        ])
        .output()
        .context("failed to invoke \"git add -A\"")?;

    if !git_commit_output.status.success() {
        let git_error = String::from_utf8_lossy(&git_add_output.stderr);
        anyhow::bail!("`git commit` exited with error:\n\n{}", git_error);
    }

    Ok(())
}
