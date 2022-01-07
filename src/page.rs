use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use git2::Status;
use pulldown_cmark::Parser;
use minijinja::{Environment, Source};

/// Converts markdown to sanitized html.
pub fn markdown_to_html(markdown: &str) -> String {
    let parser = markdown_parser(markdown);
    let mut page_html_unsafe = String::new();
    pulldown_cmark::html::push_html(&mut page_html_unsafe, parser);
    ammonia::clean(&page_html_unsafe)
}

pub fn markdown_parser(markdown: &str) -> Parser {
    let options = pulldown_cmark::Options::all();
    Parser::new_ext(markdown, options)
}

/// Extracts the first title found in markdown's syntax.
pub fn extract_title(page: &str) -> String {
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
            },
            _ => {},
        }
    }

    String::from("Untitled")
}

pub fn read_theme_templates(project: &Path) -> Result<Environment> {
    let templates_dir = project.join(".codasai/theme/templates");

    let mut engine = Environment::new();
    let mut source = Source::new();
    source.load_from_path(&templates_dir, &["html"])?;
    engine.set_source(source);

    fn url_join(_state: &minijinja::State, base_url: String, fragment: String) -> Result<String, minijinja::Error> {
        let fragment = Path::new(&fragment);
        let relative_fragment = fragment.strip_prefix("/").unwrap_or(fragment);

        Ok(Path::new(&base_url)
            .join(relative_fragment)
            .display()
            .to_string()
            .into())
    }
    engine.add_filter("url_join", url_join);

    Ok(engine)
}

/// Find the unsaved page in the project.
///
/// It uses `git status` to detect what page is new.
///
/// It returns an error if there are multiple unsaved pages.
pub fn find_unsaved_page(project: &Path) -> Result<Option<PathBuf>> {
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

    Ok(page)
}
