use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct File {
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
