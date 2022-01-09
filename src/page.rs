use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use git2::Status;
use minijinja::{Environment, Source};
use pulldown_cmark::Parser;

use crate::context::GuideContext;

/// Structure used to preprocess markdown files.
///
/// This structure gives markdown files more capabilities by preprocessing the contents with a
/// template engine that exposes useful functions.
pub struct PagePreprocessor<'a> {
    env: Environment<'a>,
}

impl<'a> PagePreprocessor<'a> {
    pub fn new(ctx: &'a GuideContext) -> Self {
        let mut env = Environment::new();

        let url = ctx.base_url.clone();
        let static_resource = move |_: &minijinja::State, path: String| -> Result<String, minijinja::Error> {
            let mut url = url.clone();

            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("public/user/");

            let relative_path = path.strip_prefix('/').unwrap_or(&path);

            url.push_str(relative_path);

            Ok(url)
        };

        env.add_function("static_resource", static_resource);

        Self { env }
    }

    pub fn preprocess(&self, name: &str, page: &str) -> Result<String> {
        let mut env = self.env.clone();
        env.add_template(name, page)?;
        let out = env.get_template(name).unwrap().render(&())?;

        Ok(out)
    }
}

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
            }
            _ => {}
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

    fn url_join(
        _state: &minijinja::State, base_url: String, fragment: String,
    ) -> Result<String, minijinja::Error> {
        let fragment = Path::new(&fragment);
        let relative_fragment = fragment.strip_prefix("/").unwrap_or(fragment);

        Ok(Path::new(&base_url)
            .join(relative_fragment)
            .display()
            .to_string())
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
