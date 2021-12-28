use serde::Serialize;

use super::Index;

#[derive(Serialize)]
pub struct GuideContext {
    pub index: Index,
    pub base_url: String,
}
