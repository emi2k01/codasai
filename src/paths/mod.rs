use std::path::{PathBuf, Path};

use anyhow::{Context, Result};

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
