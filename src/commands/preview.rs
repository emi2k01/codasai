use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use git2::Status;
use once_cell::unsync::Lazy;
use pulldown_cmark::Parser;
use serde::Serialize;
use structopt::StructOpt;
use syntect::{
    highlighting::ThemeSet,
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};
use tera::Tera;
use walkdir::WalkDir;

thread_local! {
    static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
        SyntaxSet::load_defaults_newlines()
    });
    static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
        ThemeSet::load_defaults()
    });
}

#[derive(StructOpt)]
pub struct Opts {
    #[structopt(short, long)]
    _path: Option<PathBuf>,
    #[structopt(short, long)]
    open: bool,
}

#[derive(Serialize)]
struct File {
    name: String,
    depth: i32,
    url: String,
}

impl File {
    fn new(name: String, depth: i32, url: String) -> Self {
        Self { name, depth, url }
    }
}

#[derive(Serialize)]
struct Directory {
    name: String,
    depth: i32,
    directories: Vec<Directory>,
    files: Vec<File>,
}

impl Directory {
    fn new(name: String, depth: i32) -> Self {
        Self {
            name,
            depth,
            directories: Vec::new(),
            files: Vec::new(),
        }
    }
}

#[derive(Serialize)]
struct PageContext {
    title: String,
    content: String,
    workspace: Directory,
}

pub fn execute(opts: &Opts) -> Result<()> {
    //TODO: Take `--path` into account
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

    copy_user_public_dir(&project).context("failed to export public directory")?;
    copy_theme_public_dir(&project).context("failed to export theme public directory")?;
    compile_sass(&project).context("failed to render sass files")?;
    render_workspace(&project).context("failed to render workspace")?;

    let template_engine = read_templates(&project)?;
    render_page(&project, template_engine).context("failed to render page")?;

    launch_server(&export_dir, opts.open);

    Ok(())
}

fn launch_server(export_dir: &Path, open: bool) {
    tokio::runtime::Builder::new_current_thread()
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

fn compile_sass(project: &Path) -> Result<()> {
    let sass_dir = project.join(".codasai/theme/sass");
    let out_dir = project.join(".codasai/export/public/style");

    let walkdir = WalkDir::new(&sass_dir)
        .into_iter()
        .filter_map(|entry| {
            if let Err(e) = &entry {
                log::warn!("failed to read entry {:?}", e);
            }
            entry.ok()
        })
        .filter(|entry| {
            entry.path().extension() == Some(OsStr::new("scss"))
                // ignore scss partials
                && !entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("_")
        });

    for entry in walkdir {
        if entry.metadata().map(|m| m.is_file()).unwrap_or(false) {
            let path = entry.path();
            let compiled_sass = sass_rs::compile_file(path, sass_rs::Options::default())
                // `compile_file` returns an error that doesn't implement `std::error::Error` -.-
                .map_err(|e| anyhow::anyhow!("{}", e))
                .with_context(|| format!("failed to compile sass file {:?}", path))?;

            // Put the compiled SASS files under `out_dir` following the same directory structure they had in `sass_dir`
            // that is, reuse the hierarchy in brackets in the line below vvv
            // .codasai/sass/[header/style.scss] -> .codasai/export/preview/public/style/[header/style.css]
            let relative_path = entry.path().strip_prefix(&sass_dir)?;
            let mut out_path = out_dir.join(&relative_path);
            out_path.set_extension("css");
            let parent_dir = out_path.parent().unwrap();

            anyhow::ensure!(!out_path.exists(), "file already exists {:?}", &out_path);

            std::fs::create_dir_all(&parent_dir)
                .with_context(|| format!("failed to create directory {:?}", parent_dir))?;

            std::fs::write(&out_path, &compiled_sass)
                .with_context(|| format!("failed to write to {:?}", out_path))?;
        }
    }

    Ok(())
}

fn copy_theme_public_dir(project: &Path) -> Result<()> {
    let public = project.join(".codasai/theme/public");
    let dest = project.join(".codasai/export/public/theme");

    copy_dir_contents(&public, &dest)
}

fn copy_user_public_dir(project: &Path) -> Result<()> {
    let public = project.join("public");
    let dest = project.join(".codasai/export/public");

    copy_dir_contents(&public, &dest)
}

fn copy_dir_contents(dir: &Path, dest: &Path) -> Result<()> {
    let walkdir = WalkDir::new(&dir).into_iter().filter_map(|entry| {
        if let Err(e) = &entry {
            log::warn!("failed to read entry {:?}", e);
        }
        entry.ok()
    });

    for entry in walkdir {
        if entry.metadata().map(|m| m.is_file()).unwrap_or(false) {
            let relative_path = entry.path().strip_prefix(&dir)?;
            let out_path = dest.join(&relative_path);
            let parent = out_path.parent().unwrap();

            std::fs::create_dir_all(&parent)
                .with_context(|| format!("failed to create directory {:?}", parent))?;

            std::fs::copy(entry.path(), &out_path).with_context(|| {
                format!(
                    "failed to copy file from {:?} to {:?}",
                    entry.path(),
                    out_path
                )
            })?;
        }
    }

    Ok(())
}

fn render_page(project: &Path, template_engine: Tera) -> Result<()> {
    let repo = git2::Repository::open(project)
        .with_context(|| format!("failed to open Git repository at {:?}", project))?;
    let statuses = repo.statuses(None).context("failed to get Git status")?;

    let mut page = None;
    for status in statuses.iter() {
        let path = status.path().ok_or(anyhow::anyhow!(
            "path is not valid utf-8: {:?}",
            String::from_utf8_lossy(status.path_bytes())
        ))?;

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
    // `page` as given by git2 is relative to the git repository root but we need the absolute path.
    let page = project.join(page);
    let page =
        std::fs::read_to_string(&page).with_context(|| format!("failed to read {:?}", &page))?;

    let title = escape_html(&extract_title_from_page(&page));

    let parser = markdown_parser(&page);
    let mut page_html_unsafe = String::new();
    pulldown_cmark::html::push_html(&mut page_html_unsafe, parser);
    let page_html = ammonia::clean(&page_html_unsafe);

    let page_context = PageContext {
        title,
        content: page_html,
        workspace: build_workspace_tree(&project)?,
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

fn build_workspace_tree(project: &Path) -> Result<Directory> {
    let workspace = project.join("workspace");

    let mut directories = vec![Directory::new(String::new(), 0)];
    let mut depth = 0;

    let walker = WalkDir::new(&workspace)
        .into_iter()
        .filter_map(|entry| {
            if let Err(e) = &entry {
                log::warn!("failed to read entry {:?}", e);
            }
            entry.ok()
        })
        .skip(1); // skip `workspace/`

    for entry in walker {
        if entry.file_type().is_dir() {
            if entry.depth() <= depth {
                let last_dir = directories.pop().unwrap();
                directories.last_mut().unwrap().directories.push(last_dir);
            }
            directories.push(Directory::new(
                entry.file_name().to_str().unwrap().to_string(),
                entry.depth() as i32,
            ));
        } else if entry.file_type().is_file() {
            if entry.depth() < depth {
                let last_dir = directories.pop().unwrap();
                directories.last_mut().unwrap().directories.push(last_dir);
            }
            let url = Path::new("/preview/workspace")
                .join(entry.path().strip_prefix(&workspace).unwrap())
                .display()
                .to_string();
            directories.last_mut().unwrap().files.push(File::new(
                entry.file_name().to_str().unwrap().to_string(),
                entry.depth() as i32,
                url,
            ))
        }
        depth = entry.depth();
    }

    Ok(directories.pop().unwrap())
}

fn render_workspace(project: &Path) -> Result<()> {
    let workspace = project.join("workspace");

    let walker = WalkDir::new(&workspace).into_iter().filter_map(|entry| {
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
    if let Some(extension) = file.extension().and_then(|e| e.to_str()) {
        SYNTAX_SET.with(|ss| -> Result<()> {
            if let Some(syntax) = ss.find_syntax_by_extension(extension) {
                let highlighted_code = highlight_code(&code_unsafe, syntax, &ss);
                std::fs::write(&preview_path, highlighted_code)
                    .with_context(|| format!("failed to write to {:?}", &preview_path))?;
            }

            Ok(())
        })?
    } else {
        let code = escape_html(&code_unsafe);
        std::fs::write(&preview_path, &code)
            .with_context(|| format!("failed to write to {:?}", &preview_path))?;
    };

    Ok(())
}

fn read_templates(project: &Path) -> Result<Tera> {
    let templates_dir = project.join(".codasai/theme/templates");
    let mut templates_glob = templates_dir
        .to_str()
        .ok_or(anyhow::anyhow!("templates path is not valid UTF-8"))?
        .to_string();
    templates_glob.push_str("/*.html");

    Tera::new(&templates_glob).context("failed to build template engine")
}

fn markdown_parser(markdown: &str) -> Parser {
    let options = pulldown_cmark::Options::all();
    Parser::new_ext(markdown, options)
}

fn extract_title_from_page(page: &str) -> String {
    use pulldown_cmark::{Event, Tag};

    let parser = markdown_parser(page);
    let mut in_heading = false;
    for event in parser {
        match event {
            Event::Start(Tag::Heading(_)) => in_heading = true,
            Event::End(Tag::Heading(_)) => in_heading = false,
            Event::Text(text) => {
                if in_heading {
                    return text.to_string();
                }
            }
            _ => {}
        }
    }

    return String::from("Untitled");
}

fn escape_html(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\'' => escaped.push_str("&#39;"),
            '\"' => escaped.push_str("&quot;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Renders the code to HTML with highlighting spans
///
/// # IMPORTANT
///
/// This function should always escape `code`. If `syntect` is changed for
/// another library, make sure to escape that library escapes the given code or escape it beforehand.
fn highlight_code(code: &str, syntax: &SyntaxReference, syntax_set: &SyntaxSet) -> String {
    let mut html_generator =
        ClassedHTMLGenerator::new_with_class_style(syntax, syntax_set, ClassStyle::Spaced);

    for line in LinesWithEndings::from(code) {
        html_generator.parse_html_for_line_which_includes_newline(line);
    }

    html_generator.finalize()
}
