use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub struct ProjectPaths {
    pub project: PathBuf,
    pub codasai: PathBuf,
    pub pages: PathBuf,
    pub workspace: PathBuf,
    pub user_static: PathBuf,
    pub theme: PathBuf,
    pub export: PathBuf,
    pub config_file: PathBuf,
    pub index_file: PathBuf,
}

impl ProjectPaths {
    pub fn new() -> Result<Self> {
        let project = project()?;
        let codasai = project.join(".codasai");

        Ok(Self {
            project: project.clone(),
            codasai: codasai.clone(),
            pages: project.join("pages"),
            workspace: project.join("workspace"),
            user_static: project.join("static"),
            theme: codasai.join("theme"),
            export: codasai.join("export"),
            config_file: codasai.join("guide.toml"),
            index_file: codasai.join("index.toml"),
        })
    }
}

pub fn project() -> Result<PathBuf> {
    let mut current_path = Path::new(".")
        .canonicalize()
        .with_context(|| "failed to canonicalize current directory")?;

    let mut current_dir = std::fs::read_dir(&current_path)
        .with_context(|| format!("failed to read directory {:?}", current_path))?;

    loop {
        for entry in current_dir.filter_map(Result::ok) {
            if entry.file_name() == ".codasai" && matches!(entry.file_type(), Ok(f) if f.is_dir()) {
                return Ok(current_path);
            }
        }

        if let Some(this_dir_parent) = current_path.parent() {
            current_path = this_dir_parent.to_path_buf();
        } else {
            break;
        }

        current_dir = std::fs::read_dir(&current_path)
            .with_context(|| format!("failed to read directory {:?}", current_path))?;
    }

    anyhow::bail!("failed to find a codasai project")
}
