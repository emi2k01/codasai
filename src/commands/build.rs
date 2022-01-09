use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use crate::context::{
    Directory, GlobalContext, GuideContext, Index, PageContext, WorkspaceOutlineBuilder,
};
use crate::page::PagePreprocessor;
use crate::paths;

#[derive(Parser)]
pub struct Opts {
    /// Indicates under what url the exported files will be. Useful for sites like Github/Gitlab
    /// pages
    ///
    /// By default, it is the current domain's root.
    #[clap(long, default_value = "/")]
    base_url: String,

    /// Directory where output files will be stored.
    #[clap(long)]
    export_dir: Option<PathBuf>,
}

pub fn execute(opts: &Opts) -> Result<()> {
    let mut project_paths = paths::ProjectPaths::new()?;
    if let Some(export_dir) = opts.export_dir.clone() {
        project_paths.set_export(export_dir);
    }
    let project = project_paths.project().clone();

    crate::export::export_public_files(&project_paths)?;
    let index = Index::from_project(&project)?;
    let guide_ctx = GuideContext {
        index: index.clone(),
        base_url: opts.base_url.clone(),
    };
    let preprocessor = PagePreprocessor::new(&guide_ctx);

    let repo = git2::Repository::open(&project)
        .with_context(|| format!("failed to open repository at {:?}", &project))?;

    let mut page_num = 0;
    let mut last_rev = None;
    for rev in revwalk(&repo)? {
        let rev = rev.context("failed to retrieve rev")?;

        let (file_name, page) = if let Some((name, page)) = find_new_page(&repo, last_rev, rev)? {
            (name, page)
        } else {
            continue;
        };

        let tree = repo.find_commit(rev)?.tree()?;
        let workspace_outline =
            build_workspace_outline(&repo, &tree).context("failed to build workspace outline")?;
        let page_ctx = PageContext {
            number: page_num,
            title: crate::page::extract_title(&page),
            code: index.entries[page_num].code.clone(),
            content: crate::page::markdown_to_html(&preprocessor.preprocess(&file_name, &page)?),
            workspace: workspace_outline,
            previous_page_code: index
                .entries
                .get(page_num.wrapping_sub(1))
                .map(|e| e.code.clone()),
            next_page_code: index
                .entries
                .get(page_num.wrapping_add(1))
                .map(|e| e.code.clone()),
        };

        let out_dir = project_paths.export().join(&index.entries[page_num].code);

        let context = GlobalContext {
            page: &page_ctx,
            guide: &guide_ctx,
        };
        export_page(&context, &project, &out_dir)?;

        let workspace_dir = out_dir.join("workspace");
        export_workspace(&repo, &tree, &workspace_dir)?;

        last_rev = Some(rev);
        page_num += 1;
    }

    Ok(())
}

/// Creates a [`Revwalk`](git2::Revwalk) that iterates on reverse.
///
/// It iterates by yielding the oldest revisions first.
fn revwalk(repo: &git2::Repository) -> Result<git2::Revwalk> {
    let mut revwalk = repo
        .revwalk()
        .with_context(|| "failed to create rev walker for repository")?;
    revwalk.set_sorting(git2::Sort::REVERSE)?;
    revwalk
        .push_head()
        .context("failed to push repository head")?;
    Ok(revwalk)
}

/// Finds the new page in `new_rev` by performing a diff between both revisions.
///
/// An `old_rev` with a `None` value indicates an empty tree.
///
/// Returns a tuple containing the file name and its contents
fn find_new_page(
    repo: &git2::Repository, old_rev: Option<git2::Oid>, new_rev: git2::Oid,
) -> Result<Option<(String, String)>> {
    let old_tree = old_rev.and_then(|old_rev| {
        repo.find_commit(old_rev)
            .ok()
            .and_then(|commit| commit.tree().ok())
    });
    let new_tree = repo.find_commit(new_rev)?.tree()?;

    let diff = repo.diff_tree_to_tree(old_tree.as_ref(), Some(&new_tree), None)?;

    for delta in diff.deltas() {
        if delta.status() == git2::Delta::Added {
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
                return Ok(Some((file_path.display().to_string(), String::from_utf8(page_bytes).unwrap())));
            }
        }
    }

    Ok(None)
}

/// Exports the whole workspace in the given `tree` to `workspace`.
///
/// `workspace` is a directory in the working directory and not in a git revision.
///
/// This will highlight the exported files that are supported by the highlighting engine.
fn export_workspace(repo: &git2::Repository, tree: &git2::Tree, workspace: &Path) -> Result<()> {
    std::fs::create_dir_all(workspace)
        .with_context(|| format!("failed to create directory {:?}", workspace))?;

    tree.walk(git2::TreeWalkMode::PreOrder, |parent, entry| {
        let path = Path::new(parent).join(entry.name().unwrap());
        if path.starts_with("workspace")
            && entry.to_object(repo).unwrap().kind() == Some(git2::ObjectType::Blob)
        {
            let relative_path = path.strip_prefix("workspace").unwrap();
            let mut out_path = workspace.join(&relative_path);

            let new_ext = if let Some(ext) = out_path.extension() {
                let mut new_ext = ext.to_os_string();
                new_ext.push(".html");
                new_ext
            } else {
                OsString::from("html")
            };
            out_path.set_extension(new_ext);

            std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();

            let object = entry.to_object(repo).unwrap();
            let blob = object.as_blob().unwrap();
            if blob.is_binary() {
                std::fs::write(&out_path, "BINARY FILE").unwrap();
            } else {
                let content_unsafe = String::from_utf8(blob.content().to_vec()).unwrap();
                let ext = relative_path.extension().unwrap_or_default();
                let content = crate::code::escape_and_highlight(
                    &content_unsafe,
                    ext.to_string_lossy().as_ref(),
                );

                std::fs::write(&out_path, &content).unwrap();
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    Ok(())
}

/// Exports the page with the given contexts to `out_dir/index.html`
fn export_page(ctx: &GlobalContext, project: &Path, out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create dir {:?}", &out_dir))?;

    let templates =
        crate::page::read_theme_templates(project).context("failed to read templates")?;

    let output_html = templates
        .get_template("template.html")?
        .render(&ctx)
        .context("failed to render template")?;

    let out_path = out_dir.join("index.html");
    std::fs::write(&out_path, output_html)
        .with_context(|| format!("failed to write to {:?}", &out_path))?;

    Ok(())
}

/// Traverses the workspace directory in the given `tree` and builds and outline.
fn build_workspace_outline(repo: &git2::Repository, tree: &git2::Tree) -> Result<Directory> {
    let mut ws_builder = WorkspaceOutlineBuilder::new();
    tree.walk(git2::TreeWalkMode::PreOrder, |parent, entry| {
        let parent = Path::new(parent);
        let path = parent.join(entry.name().unwrap());

        // we subtract one so that `workspace/` has depth 0, `workspace/something.txt`
        // has depth 1, etc
        let depth = path.components().count() - 1;

        if path.starts_with("workspace") && depth > 0 {
            // if the entry is a file
            if entry
                .to_object(repo)
                .map(|o| o.kind() == Some(git2::ObjectType::Blob))
                == Ok(true)
            {
                ws_builder.push_file(
                    entry.name().unwrap().to_string(),
                    path.strip_prefix("workspace")
                        .unwrap()
                        .display()
                        .to_string(),
                    depth as i32,
                );
            } else {
                ws_builder.push_dir(entry.name().unwrap().to_string(), depth as i32);
            }
        }

        git2::TreeWalkResult::Ok
    })?;

    Ok(ws_builder.finish())
}
