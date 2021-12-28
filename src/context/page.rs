use serde::Serialize;

use super::Directory;

#[derive(Serialize)]
pub struct PageContext {
    pub number: usize,
    pub title: String,
    pub code: String,
    pub content: String,
    pub workspace: Directory,
    pub previous_page_code: Option<String>,
    pub next_page_code: Option<String>,
}
