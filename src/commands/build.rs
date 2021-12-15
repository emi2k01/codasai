use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use structopt::StructOpt;

use crate::paths;

#[derive(StructOpt)]
pub struct Opts {}

pub fn execute(opts: &Opts) -> Result<()> {
    let project = paths::project()?;
    let repo = git2::Repository::open(&project)
        .with_context(|| format!("failed to open repository at {:?}", &project))?;

    let mut revwalk = repo.revwalk().with_context(|| {
        format!(
            "failed to create rev walker for repository at {:?}",
            &project
        )
    })?;
    revwalk.set_sorting(git2::Sort::REVERSE)?;
    revwalk
        .push_head()
        .context("failed to push repository head")?;

    let mut page_num = 0;

    for rev in revwalk {
        let rev = rev.context("failed to retrieve rev")?;
        let tree = repo.find_commit(rev)?.tree()?;

        let mut is_rev_relevant = false;
        tree.walk(git2::TreeWalkMode::PreOrder, |parent, entry| {
            let path = Path::new(parent).join(entry.name().expect("expected a UTF-8 valid name"));

            if path.starts_with("workspace") || path.starts_with("pages") {
                is_rev_relevant = true;
            }

            if path.starts_with("workspace") {
                let out_root = PathBuf::from(format!(".codasai/export/{}/", page_num));
                let out_path = out_root.join(path);
                std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();
            }

            git2::TreeWalkResult::Ok
        })?;

        if is_rev_relevant {
            page_num += 1;
        }
    }

    Ok(())
}

fn find_first_page(repo: &git2::Repository, rev: git2::Oid) -> Result<String> {
    let tree = repo.find_commit(rev)?.tree()?;

    let mut page = None;
    tree.walk(git2::TreeWalkMode::PreOrder, |parent, entry| {
        let path = Path::new(parent).join(entry.name().expect("expected a valid UTF-8 valid name"));

        if path.starts_with("pages") {
            page = Some(
                String::from_utf8(
                    entry
                        .to_object(repo)
                        .unwrap()
                        .as_blob()
                        .unwrap()
                        .content()
                        .to_vec(),
                )
                .unwrap(),
            );
            return git2::TreeWalkResult::Abort;
        }
        git2::TreeWalkResult::Ok
    })?;

    page.ok_or(anyhow::anyhow!("failed to find page in commit"))
}

fn find_new_page(
    repo: &git2::Repository, old_rev: git2::Oid, new_rev: git2::Oid,
) -> Result<Option<String>> {
    let old_tree = repo.find_commit(old_rev)?.tree()?;
    let new_tree = repo.find_commit(new_rev)?.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&old_tree), None)?;
    for delta in diff.deltas() {
        match delta.status() {
            git2::Delta::Added => {
                let file = delta.new_file();
                let file_path = file.path().unwrap();
                if file_path.starts_with("pages") {
                    let page_bytes = new_tree
                        .get_path(file_path)?
                        .to_object(repo)?
                        .as_blob()
                        .unwrap()
                        .content()
                        .to_vec();
                    return Ok(Some(String::from_utf8(page_bytes).unwrap()));
                }
            },
            _ => {},
        }
    }

    Ok(None)
}

fn render_workspace(out_dir: &Path, tree: git2::Tree) -> Result<()> {
    todo!()
}
