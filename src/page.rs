use std::path::Path;

use anyhow::{Context, Result};
use pulldown_cmark::Parser;
use serde::Serialize;
use tera::Tera;

use crate::exporter::Directory;

#[derive(Serialize)]
pub struct PageContext {
    pub title: String,
    pub content: String,
    pub workspace: Directory,
    pub root_url: String,
    pub page_url: String,
    pub previous_page: i32,
    pub next_page: i32,
}

pub fn to_html(markdown: &str) -> String {
    let parser = markdown_parser(&markdown);
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

    return String::from("Untitled");
}

pub fn read_templates(project: &Path) -> Result<Tera> {
    let templates_dir = project.join(".codasai/theme/templates");
    let mut templates_glob = templates_dir
        .to_str()
        .ok_or(anyhow::anyhow!("templates path is not valid UTF-8"))?
        .to_string();
    templates_glob.push_str("/*.html");

    Tera::new(&templates_glob).context("failed to build template engine")
}
