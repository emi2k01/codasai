use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use git2::Status;
use pulldown_cmark::Parser;
use serde::Serialize;
use tera::Tera;

use crate::exporter::Directory;

#[derive(Serialize)]
pub struct PageContext {
    pub title: String,
    pub content: String,
    pub workspace: Directory,
    pub base_url: String,
    pub page_url: String,
    pub previous_page: i32,
    pub next_page: i32,
}

pub fn to_html(markdown: &str) -> String {
    let parser = markdown_parser(markdown);
    let mut page_html_unsafe = String::new();
    pulldown_cmark::html::push_html(&mut page_html_unsafe, parser);
    ammonia::clean(&page_html_unsafe)
}

pub fn markdown_parser(markdown: &str) -> Parser {
    let options = pulldown_cmark::Options::all();
    Parser::new_ext(markdown, options)
}

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

pub fn read_templates(project: &Path) -> Result<Tera> {
    use std::collections::HashMap;

    use tera::Value;

    let templates_dir = project.join(".codasai/theme/templates");
    let mut templates_glob = templates_dir
        .to_str()
        .ok_or(anyhow::anyhow!("templates path is not valid UTF-8"))?
        .to_string();
    templates_glob.push_str("/*.html");

    let mut engine = Tera::new(&templates_glob).context("failed to build template engine")?;

    engine.register_filter(
        "url_join",
        |base_url: &Value, args: &HashMap<String, Value>| {
            let base_url = match base_url {
                Value::String(ref v) => v.clone(),
                Value::Number(ref v) => v.to_string(),
                _ => return Err(tera::Error::msg("expected base url to be of type `string`")),
            };

            let with = match args.get("with") {
                Some(&Value::String(ref v)) => v.clone(),
                Some(&Value::Number(ref v)) => v.to_string(),
                Some(_) => {
                    return Err(tera::Error::msg(
                        "expected argument `with` to be of type `string` or `number`",
                    ))
                },
                None => {
                    return Err(tera::Error::msg(
                        "expected argument `with` of type `string` or `number`",
                    ))
                },
            };

            let with = Path::new(&with);
            let relative_with = if with.has_root() {
                with.strip_prefix("/")
                    .expect("path should've started with root")
            } else {
                with
            };

            Ok(Path::new(&base_url)
                .join(relative_with)
                .display()
                .to_string()
                .into())
        },
    );

    Ok(engine)
}

pub fn find_new_page(project: &Path) -> Result<Option<PathBuf>> {
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
