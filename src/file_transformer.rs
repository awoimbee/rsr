use std::borrow::Cow;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;

#[derive(Debug)]
pub struct FileTransformer<'a> {
    // path: PathBuf,
    /// Is referenced by self.new_contents
    contents: Pin<String>,
    /// References self.contents
    new_contents: Vec<Cow<'a, str>>,
    modified: bool,
}

impl<'a> FileTransformer<'a> {
    pub fn new(file_name: &Path) -> Option<Self> {
        let mut contents = String::new();
        let mut f = match File::open(file_name) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Could not open {} ({})", file_name.to_string_lossy(), e);
                return None;
            }
        };
        match f.read_to_string(&mut contents) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != std::io::ErrorKind::InvalidData {
                    eprintln!("Could not read {} ({})", file_name.to_string_lossy(), e);
                }
                return None;
            }
        };
        let contents = Pin::new(contents);
        // Here, transmute hides to the compileer the fact that new_contents -> contents
        let new_contents = vec![Cow::from(unsafe {
            std::mem::transmute::<&str, &str>(&contents)
        })];

        Some(FileTransformer {
            contents,
            new_contents,
            modified: false,
        })
    }
    pub fn reader_replace(&mut self, re_start: usize, re_end: usize, replacement: String) {
        let before = re_start;
        let after = re_end;
        // txt has the same lifetime as FileTransformer. You know it, I know it, Ructc should know it.
        let txt: &'a str = unsafe { std::mem::transmute(&self.new_contents.pop().unwrap()[..]) };

        if before != 0 {
            self.new_contents.push(Cow::from(&txt[..before]));
        }
        self.new_contents.push(Cow::from(replacement));
        self.new_contents.push(Cow::from(&txt[after..]));
        self.modified = true;
    }
    pub fn reset_reader(&mut self) {
        // recreating a Pin here is ok since new_contents is drained
        self.contents = Pin::new(self.new_contents.drain(..).map(|s| s).collect());
        let txt: &'a str = unsafe { std::mem::transmute(&self.contents[..]) };
        self.new_contents.push(Cow::from(txt));
    }
    pub fn reader(&self) -> &str {
        &self.new_contents[self.new_contents.len() - 1]
    }

    /// FileTransformer shall not be reused after a call to this
    /// TODO: poison FileTransformer
    pub fn write_file(&mut self, file: &Path) -> bool {
        let open_options = OpenOptions::new().write(true).truncate(true).open(file);
        let mut file_w = match open_options {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Could not open {} for writing ({})",
                    file.to_string_lossy(),
                    e
                );
                return false;
            }
        };

        match file_w.write(self.new_contents.drain(..).collect::<String>().as_bytes()) {
            Ok(_size) => true,
            Err(e) => {
                eprintln!("Could write to {} ({})", file.to_string_lossy(), e);
                false
            }
        }
    }
}
