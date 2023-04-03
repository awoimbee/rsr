use std::borrow::Cow;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::mem::transmute;
use std::path::Path;
use std::pin::Pin;

#[derive(Debug)]
pub struct FileTransformer<'a> {
    txt: Pin<String>,
    unread_txt: &'a str,
    new_txt: Vec<Cow<'a, str>>,
    modified: bool,
    path: &'a Path,
}

impl<'a> FileTransformer<'a> {
    pub fn new(file_name: &'a Path) -> Option<Self> {
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
        let contents_ref: &'a str = unsafe { transmute(&*contents) };

        Some(FileTransformer {
            txt: contents,
            new_txt: Vec::with_capacity(100),
            unread_txt: contents_ref,
            modified: false,
            path: file_name,
        })
    }
    /// Move the read needle forward, discarding text
    pub fn reader_skip(&mut self, amount: usize) {
        self.unread_txt = &self.unread_txt[amount..];
    }
    /// Move the read needle forward, saving the text it skims
    pub fn reader_push(&mut self, amount: usize) {
        let (before, after) = self.unread_txt.split_at(amount);
        self.new_txt.push(Cow::from(before));
        self.unread_txt = after;
    }
    /// Push new text into the file where the read head sits
    pub fn push(&mut self, new: Cow<'a, str>) {
        self.new_txt.push(new);
    }

    /// Reset the read head
    pub fn reset_reader(&mut self) {
        if self.new_txt.is_empty() {
            return;
        };
        self.modified = true;
        self.new_txt.push(Cow::from(self.unread_txt));
        self.txt = Pin::new(self.new_txt.drain(..).collect());
        self.unread_txt = unsafe { transmute(&*self.txt) };
    }

    pub fn get_reader(&self) -> &'a str {
        self.unread_txt
    }

    /// Write to disk
    pub fn commit(mut self) -> bool {
        self.reset_reader();
        if !self.modified {
            return true;
        };
        let open_options = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.path);
        let mut file_w = match open_options {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Could not open {} for writing ({})",
                    self.path.to_string_lossy(),
                    e
                );
                return false;
            }
        };

        match file_w.write(self.unread_txt.as_bytes()) {
            Ok(_size) => true,
            Err(e) => {
                eprintln!("Could write to {} ({})", self.path.to_string_lossy(), e);
                false
            }
        }
    }
}
