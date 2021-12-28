use serde::Serialize;

use super::Index;

/// Context used to pass a guide's data to the front-end
#[derive(Serialize)]
pub struct GuideContext {
    pub index: Index,
    pub base_url: String,
}
