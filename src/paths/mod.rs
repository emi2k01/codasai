use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[allow(unused)]
pub struct ProjectPaths {
    project: PathBuf,
    codasai: PathBuf,
    pages: PathBuf,
    workspace: PathBuf,
    user_static: PathBuf,
    theme: PathBuf,
    export: PathBuf,
    config_file: PathBuf,
    index_file: PathBuf,
}

#[allow(unused)]
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

    pub fn set_export(&mut self, path: PathBuf) {
        self.export = path;
    }

    /// Get a reference to the project paths's project.
    pub fn project(&self) -> &PathBuf {
        &self.project
    }

    /// Get a reference to the project paths's codasai.
    pub fn codasai(&self) -> &PathBuf {
        &self.codasai
    }

    /// Get a reference to the project paths's pages.
    pub fn pages(&self) -> &PathBuf {
        &self.pages
    }

    /// Get a reference to the project paths's workspace.
    pub fn workspace(&self) -> &PathBuf {
        &self.workspace
    }

    /// Get a reference to the project paths's user static.
    pub fn user_static(&self) -> &PathBuf {
        &self.user_static
    }

    /// Get a reference to the project paths's theme.
    pub fn theme(&self) -> &PathBuf {
        &self.theme
    }

    /// Get a reference to the project paths's export.
    pub fn export(&self) -> &PathBuf {
        &self.export
    }

    /// Get a reference to the project paths's config file.
    pub fn config_file(&self) -> &PathBuf {
        &self.config_file
    }

    /// Get a reference to the project paths's index file.
    pub fn index_file(&self) -> &PathBuf {
        &self.index_file
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
