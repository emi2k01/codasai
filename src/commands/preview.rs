use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::Walk;
use structopt::StructOpt;
use tera::Tera;

use crate::code;
use crate::context::{Directory, GuideContext, Index, PageContext, WorkspaceOutlineBuilder};

#[derive(StructOpt)]
pub struct Opts {
    #[structopt(short, long)]
    _path: Option<PathBuf>,
    #[structopt(long)]
    no_open: bool,
    #[structopt(long)]
    no_run_server: bool,
}

pub fn execute(opts: &Opts) -> Result<()> {
    // TODO: Take `--path` into account
    let project =
        crate::paths::project().context("current directory is not in a codasai project")?;
    let export_dir = project.join(".codasai/export");
    let preview_dir = export_dir.join("preview");
    let public_dir = export_dir.join("public");

    // clean previous build
    if preview_dir.exists() {
        std::fs::remove_dir_all(&preview_dir)
            .with_context(|| format!("failed to remove directory {:?}", preview_dir))?;
    }

    crate::export::export_public_files(&project)?;
    export_workspace(&project).context("failed to render workspace")?;

    let template_engine = crate::page::read_theme_templates(&project)?;
    export_unsaved_page(&project, &template_engine).context("failed to render page")?;

    if !opts.no_run_server {
        server::launch_server(&export_dir, !opts.no_open);
    }

    Ok(())
}

/// Traverses the project's workspace and builds an outline
///
/// It respects ignore files.
fn build_workspace_outline(project: &Path) -> Result<Directory> {
    let workspace = project.join("workspace");

    let walker = Walk::new(&workspace)
        .into_iter()
        .filter_map(|entry| {
            if let Err(e) = &entry {
                log::warn!("failed to read entry {:?}", e);
            }
            entry.ok()
        })
        .skip(1); // skip `workspace/`

    let mut ws_builder = WorkspaceOutlineBuilder::new();
    for entry in walker {
        if matches!(entry.file_type(), Some(ft) if ft.is_dir()) {
            ws_builder.push_dir(
                entry.file_name().to_str().unwrap().to_string(),
                entry.depth() as i32,
            );
        } else if matches!(entry.file_type(), Some(ft) if ft.is_file()) {
            ws_builder.push_file(
                entry.file_name().to_str().unwrap().to_string(),
                entry.path().strip_prefix(&workspace)?.display().to_string(),
                entry.depth() as i32,
            );
        }
    }

    Ok(ws_builder.finish())
}

/// Exports the whole workspace in the project
///
/// It respects ignore files.
fn export_workspace(project: &Path) -> Result<()> {
    let workspace = project.join("workspace");

    let walker = Walk::new(&workspace).into_iter().filter_map(|entry| {
        if let Err(e) = &entry {
            log::warn!("failed to read entry {:?}", e);
        }
        entry.ok()
    });

    let preview_ws = project.join(".codasai/export/preview/workspace");
    if preview_ws.exists() {
        std::fs::remove_dir_all(&preview_ws)
            .with_context(|| format!("failed to remove directory {:?}", &preview_ws))?;
    }
    std::fs::create_dir_all(&preview_ws)
        .with_context(|| format!("failed to create dir {:?}", &preview_ws))?;

    for entry in walker {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                export_workspace_file(entry.path(), project, &preview_ws)
                    .with_context(|| format!("failed to render file {:?}", entry.path()))?;
            }
        }
    }

    Ok(())
}

/// Exports `file` to `preview_ws` keeping the same directory structure relative to the workspace
/// directory.
///
/// It highlights the exported files it they're supported by the highlighting engine.
fn export_workspace_file(file: &Path, project: &Path, preview_ws: &Path) -> Result<()> {
    let relative_path = file
        .strip_prefix(&project.join("workspace"))
        .expect("failed to strip prefix");
    let mut preview_path = preview_ws.join(relative_path);

    match preview_path.extension() {
        Some(extension) => {
            let mut extension = extension.to_owned();
            extension.push(".html");
            preview_path.set_extension(extension);
        }
        None => {
            preview_path.set_extension("html");
        }
    }

    let parent = preview_path.parent().unwrap();
    std::fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory {:?}", parent))?;

    let code_unsafe =
        std::fs::read_to_string(file).with_context(|| format!("failed to read file {:?}", file))?;

    // Only languages supported by `syntect` are highlighted.
    // Files that don't have a supported file extension are only escaped.
    let extension = file.extension().unwrap_or_default().to_str().unwrap();
    let code = code::escape_and_highlight(&code_unsafe, extension);
    std::fs::write(&preview_path, &code)
        .with_context(|| format!("failed to write to {:?}", &preview_path))?;

    Ok(())
}

/// Exports the unsaved page in the project.
///
/// It uses `template.html` in `template_engine` to render the page.
pub fn export_unsaved_page(project: &Path, template_engine: &Tera) -> Result<()> {
    let page = crate::page::find_unsaved_page(project).context("failed to find new page")?;
    let page = page.ok_or(anyhow::anyhow!("there are no unsaved pages"))?;
    // `page` as given by git2 is relative to the git repository root but we need
    // the absolute path.
    let page = project.join(page);
    let page =
        std::fs::read_to_string(&page).with_context(|| format!("failed to read {:?}", &page))?;

    let title = crate::page::extract_title(&page);
    let page_html = crate::page::markdown_to_html(&page);

    let guide_context = GuideContext {
        base_url: "/".to_string(),
        index: Index::default(),
    };

    let page_context = PageContext {
        number: 0,
        title,
        content: page_html,
        code: "preview".to_string(),
        workspace: build_workspace_outline(project)?,
        previous_page_code: None,
        next_page_code: None,
    };

    let mut context = tera::Context::new();
    context.insert("page", &page_context);
    context.insert("guide", &guide_context);
    let reader_html = template_engine
        .render("template.html", &context)
        .context("failed to render template")?;

    let preview = project.join(".codasai/export/preview");
    std::fs::create_dir_all(&preview)
        .with_context(|| format!("failed to create directory {:?}", preview))?;

    let reader_path = preview.join("index.html");
    std::fs::write(&reader_path, &reader_html)
        .with_context(|| format!("failed to write to {:?}", &reader_path))?;

    Ok(())
}
