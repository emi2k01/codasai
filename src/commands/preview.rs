use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use git2::Status;
use ignore::Walk;
use structopt::StructOpt;
use tera::Tera;

use crate::code;
use crate::exporter::{self, Directory, WorkspaceOutlineBuilder};
use crate::page::{self, PageContext};

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

    if public_dir.exists() {
        std::fs::remove_dir_all(&public_dir)
            .with_context(|| format!("failed to remove directory {:?}", public_dir))?;
    }

    exporter::setup_public_files(&project)?;
    render_workspace(&project).context("failed to render workspace")?;

    let template_engine = page::read_templates(&project)?;
    render_page(&project, &template_engine).context("failed to render page")?;

    if !opts.no_run_server {
        launch_server(&export_dir, !opts.no_open);
    }

    Ok(())
}

fn launch_server(export_dir: &Path, open: bool) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            let address = [127, 0, 0, 1];
            let port = 8000;
            if open {
                tokio::spawn(async move {
                    let url = format!(
                        "http://{}.{}.{}.{}:{}/preview",
                        address[0], address[1], address[2], address[3], port
                    );
                    if let Err(e) = open::that(url).context("failed to open browser") {
                        log::warn!("{}", e);
                    }
                });
            }
            warp::serve(warp::fs::dir(export_dir.to_path_buf()))
                .run((address, port))
                .await;
        });
}

fn build_workspace_tree(project: &Path) -> Result<Directory> {
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

fn render_workspace(project: &Path) -> Result<()> {
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
                render_file_to_preview(entry.path(), project, &preview_ws)
                    .with_context(|| format!("failed to render file {:?}", entry.path()))?;
            }
        }
    }

    Ok(())
}

/// Renders a file in the workspace to html and saves it under `preview_ws`
/// following the same directory structure relative to the project directory.
fn render_file_to_preview(file: &Path, project: &Path, preview_ws: &Path) -> Result<()> {
    let relative_path = file
        .strip_prefix(&project.join("workspace"))
        .expect("failed to strip prefix");
    let mut preview_path = preview_ws.join(relative_path);

    match preview_path.extension() {
        Some(extension) => {
            let mut extension = extension.to_owned();
            extension.push(".html");
            preview_path.set_extension(extension);
        },
        None => {
            preview_path.set_extension("html");
        },
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

pub fn render_page(project: &Path, template_engine: &Tera) -> Result<()> {
    let repo = git2::Repository::open(project)
        .with_context(|| format!("failed to open Git repository at {:?}", project))?;
    let statuses = repo.statuses(None).context("failed to get Git status")?;

    let mut page = None;
    for status in statuses.iter() {
        let path = status.path().ok_or_else(|| {
            anyhow::anyhow!(
                "path is not valid utf-8: {:?}",
                String::from_utf8_lossy(status.path_bytes())
            )
        })?;

        let path = PathBuf::from(path);
        if status.status() == Status::WT_NEW
            && path.starts_with("pages")
            && path.extension() == Some(OsStr::new("md"))
        {
            anyhow::ensure!(page.is_none(), "there is more that one unsaved page");
            page = Some(path);
        }
    }

    let page = page.ok_or(anyhow::anyhow!("there are no unsaved pages"))?;
    // `page` as given by git2 is relative to the git repository root but we need
    // the absolute path.
    let page = project.join(page);
    let page =
        std::fs::read_to_string(&page).with_context(|| format!("failed to read {:?}", &page))?;

    let title = page::extract_title(&page);
    let page_html = page::to_html(&page);

    let page_context = PageContext {
        title,
        content: page_html,
        workspace: build_workspace_tree(project)?,
        base_url: "/".to_string(),
        page_url: "/preview".to_string(),
        previous_page: -1,
        next_page: -1,
    };

    let mut context = tera::Context::new();
    context.insert("page", &page_context);
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
