use os_str_bytes::OsStrBytes;
use regex::bytes::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileWalker {
    file_stack: Vec<std::path::PathBuf>,
    dir_stack: Vec<std::path::PathBuf>,
    reg: Regex,
}

impl FileWalker {
    pub fn new(root: &str, regex: &str) -> Self {
        let mut file_stack = Vec::with_capacity(50);
        let mut dir_stack = Vec::with_capacity(50);
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

    fn allowed_dir(dname: &Path) -> bool {
        !dname.ends_with(".git") && !dname.ends_with("vendor") && !dname.ends_with("var/cache")
    }
}

impl Iterator for FileWalker {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        while self.file_stack.is_empty() {
            let dir = self.dir_stack.pop()?;
            let dir_reader = match fs::read_dir(&dir) {
                Ok(dr) => dr,
                Err(e) => {
                    eprintln!("Couldn't read {}: {}", dir.to_string_lossy(), e);
                    continue;
                }
            };
            for f in dir_reader {
                // use unwrap because it's not safe to continue if the folder
                // structure changes while this is running anyways
                let f = f.unwrap();
                let f_type = f.metadata().unwrap().file_type();
                let f_path = f.path();
                if f_type.is_file() && self.reg.is_match(&f_path.as_os_str().to_bytes()) {
                    self.file_stack.push(f_path);
                } else if f_type.is_dir() && Self::allowed_dir(&f_path) {
                    self.dir_stack.push(f_path);
                } else {
                    // is a symlink, do nothing
                }
            }
        }
        self.file_stack.pop()
    }
}
