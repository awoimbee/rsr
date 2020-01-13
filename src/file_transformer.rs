use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;

#[derive(Debug)]
pub struct FileTransformer {
    contents: String,
    read_ofst: usize,
}

/* General methods */
impl FileTransformer {
    pub fn new(file_name: &str) -> Option<Self> {
        let mut contents = String::new();
        let mut f = match File::open(file_name) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Could not open {} ({})", file_name, e);
                return None;
            }
        };
        match f.read_to_string(&mut contents) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != std::io::ErrorKind::InvalidData {
                    eprintln!("Could not read {} ({})", file_name, e);
                }
                return None;
            }
        };

        Some(FileTransformer {
            contents,
            read_ofst: 0,
        })
    }
    pub fn reader_replace(&mut self, re_start: usize, re_end: usize, replacement: String) {
        let before = re_start + self.read_ofst;
        let after = re_end + self.read_ofst;
        if after - before == replacement.len() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    replacement.as_bytes().as_ptr(),
                    &mut self.contents.as_bytes_mut()[before - 1],
                    replacement.len(),
                );
            }
        } else {
            self.contents = format!(
                "{}{}{}",
                &self.contents[..before],
                replacement,
                &self.contents[after..]
            );
        }
        self.read_ofst = before + replacement.len();
    }
    pub fn reset_reader(&mut self) {
        self.read_ofst = 0;
    }
    pub fn reader(&self) -> &str {
        let c_len = self.contents.len();
        match self.read_ofst < c_len {
            true => &self.contents[self.read_ofst..],
            _ if c_len != 0 => &self.contents[c_len - 1..],
            _ => &self.contents,
        }
    }
    pub fn write_file(&self, file_name: &str) -> bool {
        let open_options = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(file_name);
        let mut file_w = match open_options {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Could not open file ({})", e);
                return false;
            }
        };
        match file_w.write(self.contents.as_bytes()) {
            Ok(_size) => true,
            Err(e) => {
                eprintln!("Could write to file ({})", e);
                false
            }
        }
    }
}
