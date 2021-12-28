use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Index {
    pub entries: Vec<IndexEntry>,
}

impl Index {
    pub fn from_project(project: &Path) -> Result<Self> {
        let index_path = project.join(".codasai/index.toml");
        let index_toml = std::fs::read_to_string(&index_path)
            .with_context(|| format!("failed to read page registry {:?}", &index_path))?;
        Ok(toml::from_str(&index_toml)
            .with_context(|| format!("failed to deserialize index at {:?}", &index_path))?)
    }

    pub fn write_to_project(&self, project: &Path) -> Result<()> {
        let index_path = project.join(".codasai/index.toml");
        std::fs::write(&index_path, toml::to_string_pretty(self)?)
            .with_context(|| format!("failed to write index to {:?}", index_path))?;
        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct IndexEntry {
    pub name: String,
    pub code: String,
}
