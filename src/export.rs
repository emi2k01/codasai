use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result};
use walkdir::WalkDir;

/// Takes care of exporting all files needed by the guide such as images, css, etc.
pub fn export_public_files(project: &Path) -> Result<()> {
    let public_dir = project.join(".codasai/export/public");

    if public_dir.exists() {
        std::fs::remove_dir_all(&public_dir).context("failed to remove public dir")?;
    }

    export_user_public_dir(project).context("failed to export public directory")?;
    copy_theme_public_dir(project).context("failed to export theme public directory")?;
    compile_sass(project).context("failed to render sass files")?;

    Ok(())
}

/// Compiles the project's theme sass to the exported public directory
fn compile_sass(project: &Path) -> Result<()> {
    let sass_dir = project.join(".codasai/theme/sass");
    let out_dir = project.join(".codasai/export/public/style");

    let walkdir = WalkDir::new(&sass_dir)
        .into_iter()
        .filter_map(|entry| {
            if let Err(e) = &entry {
                log::warn!("failed to read entry {:?}", e);
            }
            entry.ok()
        })
        .filter(|entry| {
            entry.path().extension() == Some(OsStr::new("scss"))
                // ignore scss partials
                && !entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with('_')
        });

    for entry in walkdir {
        if entry.metadata().map(|m| m.is_file()).unwrap_or(false) {
            let path = entry.path();
            let compiled_sass = sass_rs::compile_file(path, sass_rs::Options::default())
                // `compile_file` returns an error that doesn't implement `std::error::Error` -.-
                .map_err(|e| anyhow::anyhow!("{}", e))
                .with_context(|| format!("failed to compile sass file {:?}", path))?;

            // Put the compiled SASS files under `out_dir` following the same directory
            // structure they had in `sass_dir` that is, reuse the hierarchy in
            // brackets in the line below vvv .codasai/sass/[header/style.scss]
            // -> .codasai/export/preview/public/style/[header/style.css]
            let relative_path = entry.path().strip_prefix(&sass_dir)?;
            let mut out_path = out_dir.join(&relative_path);
            out_path.set_extension("css");
            let parent_dir = out_path.parent().unwrap();

            anyhow::ensure!(!out_path.exists(), "file already exists {:?}", &out_path);

            std::fs::create_dir_all(&parent_dir)
                .with_context(|| format!("failed to create directory {:?}", parent_dir))?;

            std::fs::write(&out_path, &compiled_sass)
                .with_context(|| format!("failed to write to {:?}", out_path))?;
        }
    }

    Ok(())
}

/// Exports the public directory provided by the author of the project's theme.
fn copy_theme_public_dir(project: &Path) -> Result<()> {
    let public = project.join(".codasai/theme/public");
    let dest = project.join(".codasai/export/public/theme");

    copy_dir_contents(&public, &dest)
}

/// Exports the public directory provided by the author of the guide.
fn export_user_public_dir(project: &Path) -> Result<()> {
    let public = project.join("public");
    let dest = project.join(".codasai/export/public");

    copy_dir_contents(&public, &dest)
}

/// Copies all contents in `dir` to `dest` recursively.
fn copy_dir_contents(dir: &Path, dest: &Path) -> Result<()> {
    let walkdir = WalkDir::new(&dir).into_iter().filter_map(|entry| {
        if let Err(e) = &entry {
            log::warn!("failed to read entry {:?}", e);
        }
        entry.ok()
    });

    for entry in walkdir {
        if entry.metadata().map(|m| m.is_file()).unwrap_or(false) {
            let relative_path = entry.path().strip_prefix(&dir)?;
            let out_path = dest.join(&relative_path);
            let parent = out_path.parent().unwrap();

            std::fs::create_dir_all(&parent)
                .with_context(|| format!("failed to create directory {:?}", parent))?;

            std::fs::copy(entry.path(), &out_path).with_context(|| {
                format!(
                    "failed to copy file from {:?} to {:?}",
                    entry.path(),
                    out_path
                )
            })?;
        }
    }

    Ok(())
}
