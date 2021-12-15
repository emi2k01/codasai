use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result};
use structopt::StructOpt;

use crate::exporter::WorkspaceOutlineBuilder;
use crate::page::PageContext;
use crate::paths;

#[derive(StructOpt)]
pub struct Opts {}

pub fn execute(_opts: &Opts) -> Result<()> {
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

    let mut last_rev = None;
    for rev in revwalk {
        let rev = rev.context("failed to retrieve rev")?;

        let page = if let Some(last_rev) = last_rev {
            if let Some(new_page) = find_new_page(&repo, last_rev, rev)? {
                new_page
            } else {
                // if there's no new page, then we skip this revision
                continue;
            }
        } else {
            if let Some(first_page) = find_first_page(&repo, rev)? {
                first_page
            } else {
                // if there's no page, then we skip this revision
                continue;
            }
        };

        let export_dir = project.join(format!(".codasai/export/{}", page_num));
        std::fs::create_dir_all(&export_dir)
            .with_context(|| format!("failed to create dir {:?}", &export_dir))?;

        render_page(&project, &export_dir, &page)?;

        let workspace_dir = export_dir.join("workspace");
        let tree = repo.find_commit(rev)?.tree()?;
        render_workspace(&repo, &tree, &workspace_dir)?;

        last_rev = Some(rev);
        page_num += 1;
    }

    Ok(())
}

fn find_first_page(repo: &git2::Repository, rev: git2::Oid) -> Result<Option<String>> {
    let tree = repo.find_commit(rev)?.tree()?;

    let mut page = None;
    tree.walk(git2::TreeWalkMode::PreOrder, |parent, entry| {
        let path = Path::new(parent).join(entry.name().expect("expected a valid UTF-8 valid name"));

        if path.starts_with("pages") && path.extension() == Some(&OsStr::new("md")) {
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

    Ok(page)
}

fn find_new_page(
    repo: &git2::Repository, old_rev: git2::Oid, new_rev: git2::Oid,
) -> Result<Option<String>> {
    let old_tree = repo.find_commit(old_rev)?.tree()?;
    let new_tree = repo.find_commit(new_rev)?.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)?;
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

fn render_workspace(repo: &git2::Repository, tree: &git2::Tree, workspace: &Path) -> Result<()> {
    std::fs::create_dir_all(workspace)
        .with_context(|| format!("failed to create directory {:?}", workspace))?;

    tree.walk(git2::TreeWalkMode::PreOrder, |parent, entry| {
        let path = Path::new(parent).join(entry.name().unwrap());
        if path.starts_with("workspace")
            && entry.to_object(repo).unwrap().kind() == Some(git2::ObjectType::Blob)
        {
            let relative_path = path.strip_prefix("workspace").unwrap();
            let out_path = workspace.join(&relative_path);
            std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();

            let object = entry.to_object(repo).unwrap();
            let blob = object.as_blob().unwrap();
            if blob.is_binary() {
                std::fs::write(&out_path, "BINARY FILE").unwrap();
            } else {
                let content = String::from_utf8(blob.content().to_vec()).unwrap();
                std::fs::write(&out_path, &content).unwrap();
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    Ok(())
}

fn render_page(project: &Path, export_dir: &Path, page: &str) -> Result<()> {
    let title = crate::page::extract_title(&page);
    let page_html = crate::page::to_html(&page);
    let tera_engine = crate::page::read_templates(&project).context("failed to read templates")?;

    let page_context = PageContext {
        title,
        content: page_html,
        workspace: WorkspaceOutlineBuilder::new().finish(),
    };

    let mut context = tera::Context::new();
    context.insert("page", &page_context);

    let output_html = tera_engine
        .render("template.html", &context)
        .context("failed to render template")?;

    let out_path = export_dir.join("index.html");
    std::fs::write(&out_path, output_html)
        .with_context(|| format!("failed to write to {:?}", &out_path))?;

    Ok(())
}
