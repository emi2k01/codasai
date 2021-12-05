use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use once_cell::unsync::Lazy;
use structopt::StructOpt;
use syntect::{
    highlighting::ThemeSet,
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};
use walkdir::{DirEntry, WalkDir};

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
    path: Option<PathBuf>,
}

pub fn execute(_opts: &Opts) -> Result<()> {
    //TODO: Take `--path` into account
    let project =
        crate::paths::project().context("current directory is not in a codasai project")?;

    let walker = WalkDir::new(&project)
        .into_iter()
        .filter_entry(|entry| is_workspace_entry(entry, &project))
        .filter_map(|entry| {
            if let Err(e) = &entry {
                log::warn!("failed to read entry {:?}", e);
            }
            entry.ok()
        });

    let preview_ws = project.join(".codasai/preview/workspace");
    if preview_ws.exists() {
        std::fs::remove_dir_all(&preview_ws)
            .with_context(|| format!("failed to remove directory {:?}", &preview_ws))?;
    }
    std::fs::create_dir_all(&preview_ws)
        .with_context(|| format!("failed to create dir {:?}", &preview_ws))?;

    for entry in walker {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                render_file_to_preview(entry.path(), &project, &preview_ws)
                    .with_context(|| format!("failed to render file {:?}", entry.path()))?;
            }
        }
    }

    Ok(())
}

/// Renders a file in the workspace to html and saves it under `preview_ws`
/// following the same directory structure relative to the project directory.
fn render_file_to_preview(file: &Path, project: &Path, preview_ws: &Path) -> Result<()> {
    log::debug!("path {:?}", file);
    let relative_path = file.strip_prefix(&project).expect("failed to strip prefix");
    log::debug!("relative_path {:?}", relative_path);
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
    log::debug!("relative_path {:?}", preview_path);

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

fn is_workspace_entry(entry: &DirEntry, project: &Path) -> bool {
    let special_dirs = vec![
        ".codasai",
        ".git",
        "_pages",
    ];
    for dir in special_dirs {
        if entry.path().starts_with(project.join(dir)) {
            return false;
        }
    }
    return true;
}
