use serde::Serialize;

/// A structure used to hold outline information about a file
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

/// A structure used to hold outline information about a directory
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

/// A strucuture used to build a workspace outline assumming a depth-first
/// traversal of the workspace
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

    /// Push a directory into the stack.
    ///
    /// If `depth` is less than the last depth passed, it will pop the
    /// directories' stack to match `depth`.
    pub fn push_dir(&mut self, name: String, depth: i32) {
        if depth <= self.depth {
            for _ in depth..=self.depth {
                self.pop_dir();
            }
        }
        self.depth = depth;
        self.dirs.push(Directory::new(name, self.depth));
    }

    /// Pops a directory out of the directories' stack.
    ///
    /// This makes the last directory a child of the previous directory in the
    /// stack.
    fn pop_dir(&mut self) {
        let last_dir = self.dirs.pop().unwrap();
        self.dirs.last_mut().unwrap().directories.push(last_dir);
    }

    /// Pushes a file to the last pushed directory
    ///
    /// It sets the depth of the builder to the depth of the parent directory.
    /// That is, if `depth` is `3`, the builder's state will have `depth` 2.
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

    /// Finish building the outline.
    ///
    /// If there are remaining directories in the stack, they will be popped.
    ///
    /// Returns a root directory with no name that contains all the passed
    /// entries during the construction of the outline.
    pub fn finish(mut self) -> Directory {
        for _ in 0..self.depth {
            self.pop_dir();
        }
        self.dirs.pop().unwrap()
    }
}
