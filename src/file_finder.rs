use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

// The iterator starves the file modifiers threads
// The array can become way too big and slow things down
// I havent found the perfect solution

pub struct FileWalker {
    file_stack: Vec<std::path::PathBuf>,
    dir_stack: Vec<std::path::PathBuf>,
    reg: regex::Regex,
}

impl FileWalker {
    pub fn new(root: &str, regex: &str) -> Self {
        let mut file_stack = Vec::with_capacity(500);
        let mut dir_stack = Vec::with_capacity(500);
        let reg = match Regex::new(regex) {
            Ok(r) => r,
            Err(e) => panic!("Invalid regex: {}: {}", regex, e),
        };
        let r_path = PathBuf::from(root);
        let root_meta = match fs::metadata(&r_path) {
            Ok(m) => m,
            Err(e) => panic!("Invalid root dir: {} ({})", root, e),
        };
        if root_meta.is_dir() {
            dir_stack.push(r_path)
        } else {
            file_stack.push(r_path)
        };
        Self {
            file_stack,
            dir_stack,
            reg,
        }
    }

    /// TODO: use gitignore
    fn allowed_dir(dname: &Path) -> bool {
        !dname.ends_with("/.git") && !dname.ends_with("/vendor") && !dname.ends_with("/var/cache")
    }
}

impl Iterator for FileWalker {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        // while self.file_stack.is_empty() {
        while !self.dir_stack.is_empty() {
            if self.dir_stack.is_empty() {
                return None;
            };
            let dir = self.dir_stack.pop().unwrap();
            let dir_reader = match fs::read_dir(&dir) {
                Ok(dr) => dr.into_iter(),
                Err(e) => {
                    println!("Couldn't read {}: {}", dir.to_string_lossy(), e);
                    continue;
                }
            };
            for f in dir_reader {
                let f = f.unwrap(); // I dont know/understand why unwrap is necessary here
                let f_path = f.path();
                let f_meta = f.metadata().unwrap();
                if f_meta.is_file() {
                    self.file_stack.push(f_path);
                }
                // todo: rm to_string_lossy
                else if f_meta.is_dir() && Self::allowed_dir(&f_path) {
                    self.dir_stack.push(f_path);
                }
            }
        }
        self.file_stack.pop()
    }
}

/// TODO: use gitignore
pub fn allowed_dir(dname: &Path) -> bool {
    !dname.ends_with("/.git") && !dname.ends_with("/vendor") && !dname.ends_with("/var/cache")
}

/// finds files inside `root` w/ names that matches
/// Performance:
///   Good enough. It's not really slow and it permits the use of .into_par_iter()
pub fn f_find(root: &str, regex: &str) -> Vec<PathBuf> {
    let mut file_stack = Vec::new();
    let mut dir_stack = Vec::new();
    let reg = match Regex::new(regex) {
        Ok(r) => r,
        Err(e) => panic!("Invalid regex: {}: {}", regex, e),
    };
    let r_path = PathBuf::from(root);
    let root_meta = match fs::metadata(&r_path) {
        Ok(m) => m,
        Err(e) => panic!("Invalid root dir: {} ({})", root, e),
    };
    if root_meta.is_dir() {
        dir_stack.push(r_path)
    } else {
        file_stack.push(r_path)
    };

    while let Some(dir) = dir_stack.pop() {
        let dir_reader = match fs::read_dir(&dir) {
            Ok(dr) => dr,
            Err(e) => {
                println!("Couldn't read {}: {}", dir.to_string_lossy(), e);
                continue;
            }
        };
        for f in dir_reader {
            let f = f.unwrap(); // I dont know/understand why unwrap is necessary here
            let f_path = f.path();
            let f_meta = f.metadata().unwrap();
            if f_meta.is_file() {
                file_stack.push(f_path);
            }
            // todo: rm to_string_lossy
            else if f_meta.is_dir() && allowed_dir(&f_path) {
                dir_stack.push(f_path);
            }
        }
    }

    file_stack
}
