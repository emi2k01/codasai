use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
struct File {
    name: String,
    depth: i32,
    path: String,
}

impl File {
    fn new(name: String, depth: i32, path: String) -> Self {
        Self { name, depth, path }
    }
}

#[derive(Debug, Serialize)]
pub struct Directory {
    name: String,
    depth: i32,
    directories: Vec<Directory>,
    files: Vec<File>,
}

impl Directory {
    fn new(name: String, depth: i32) -> Self {
        Self {
            name,
            depth,
            directories: Vec::new(),
            files: Vec::new(),
        }
    }
}

pub struct WorkspaceOutlineBuilder {
    depth: i32,
    dirs: Vec<Directory>,
}

impl WorkspaceOutlineBuilder {
    pub fn new() -> Self {
        Self {
            depth: 0,
            dirs: vec![Directory::new(String::new(), 0)],
        }
    }

    pub fn push_dir(&mut self, name: String, depth: i32) {
        if depth <= self.depth {
            for _ in depth..=self.depth {
                self.pop_dir();
            }
        }
        self.depth = depth;
        self.dirs.push(Directory::new(name, self.depth));
    }

    fn pop_dir(&mut self) {
        let last_dir = self.dirs.pop().unwrap();
        self.dirs.last_mut().unwrap().directories.push(last_dir);
    }

    pub fn push_file(&mut self, name: String, path: String, depth: i32) {
        if depth <= self.depth {
            for _ in depth..=self.depth {
                self.pop_dir();
            }
        }
        self.depth = depth - 1;
        self.dirs
            .last_mut()
            .unwrap()
            .files
            .push(File::new(name, depth, path));
    }

    pub fn finish(mut self) -> Directory {
        for _ in 0..self.depth {
            self.pop_dir();
        }
        self.dirs.pop().unwrap()
    }
}

pub fn setup_public_files(project: &Path) -> Result<()> {
    std::fs::remove_dir_all(project.join(".codasai/export/public"))
        .context("failed to remove public dir")?;

    copy_user_public_dir(&project).context("failed to export public directory")?;
    copy_theme_public_dir(&project).context("failed to export theme public directory")?;
    compile_sass(&project).context("failed to render sass files")?;

    Ok(())
}

pub fn compile_sass(project: &Path) -> Result<()> {
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
                    .starts_with("_")
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

pub fn copy_theme_public_dir(project: &Path) -> Result<()> {
    let public = project.join(".codasai/theme/public");
    let dest = project.join(".codasai/export/public/theme");

    copy_dir_contents(&public, &dest)
}

pub fn copy_user_public_dir(project: &Path) -> Result<()> {
    let public = project.join("public");
    let dest = project.join(".codasai/export/public");

    copy_dir_contents(&public, &dest)
}

pub fn copy_dir_contents(dir: &Path, dest: &Path) -> Result<()> {
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
