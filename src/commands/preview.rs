use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use ignore::Walk;
use minijinja::Environment;

use crate::code;
use crate::context::{
    Directory, GlobalContext, GuideContext, Index, PageContext, WorkspaceOutlineBuilder,
};
use crate::page::PagePreprocessor;

#[derive(Parser)]
pub struct Opts {
    #[clap(short, long)]
    _path: Option<PathBuf>,
    /// Indicates if the browser should not be open.
    #[clap(long)]
    no_open: bool,
    /// Indicates if the built-in web server should not be used to serve the guide.
    #[clap(long)]
    no_run_server: bool,
}

pub fn execute(opts: &Opts) -> Result<()> {
    // TODO: Take `--path` into account
    let project_paths = crate::paths::ProjectPaths::new()?;
    let project = &project_paths.project;
    let export_dir = &project_paths.export;
    let preview_dir = export_dir.join("preview");

    // clean previous build
    if preview_dir.exists() {
        std::fs::remove_dir_all(&preview_dir)
            .with_context(|| format!("failed to remove directory {:?}", preview_dir))?;
    }

    crate::export::export_public_files(&project_paths)?;
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
pub fn export_unsaved_page(project: &Path, template_engine: &Environment) -> Result<()> {
    let page_path_relative = crate::page::find_unsaved_page(project)
        .context("failed to find new page")?
        .ok_or(anyhow::anyhow!("there are no unsaved pages"))?;
    // `page` as given by git2 is relative to the git repository root but we need
    // the absolute path.
    let page_path = project.join(page_path_relative);
    let page = std::fs::read_to_string(&page_path)
        .with_context(|| format!("failed to read {:?}", &page_path))?;

    let guide_context = GuideContext {
        base_url: "/".to_string(),
        index: Index::default(),
    };

    let preprocessor = PagePreprocessor::new(&guide_context);

    let page_path_str = page_path.display().to_string();
    let page_html = crate::page::markdown_to_html(
        &preprocessor.preprocess(&page_path_str, &page)?,
    );

    let title = crate::page::extract_title(&page);
    let page_context = PageContext {
        number: 0,
        title,
        content: page_html,
        code: "preview".to_string(),
        workspace: build_workspace_outline(project)?,
        previous_page_code: None,
        next_page_code: None,
    };

    let context = GlobalContext {
        page: &page_context,
        guide: &guide_context,
    };

    let reader_html = template_engine
        .get_template("template.html")?
        .render(&context)
        .context("failed to render template")?;

    let preview = project.join(".codasai/export/preview");
    std::fs::create_dir_all(&preview)
        .with_context(|| format!("failed to create directory {:?}", preview))?;

    let reader_path = preview.join("index.html");
    std::fs::write(&reader_path, &reader_html)
        .with_context(|| format!("failed to write to {:?}", &reader_path))?;

    Ok(())
}
