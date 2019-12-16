use mini_v8::{MiniV8, Function, Invocation, Error as MV8Error, Value};
use std::fs;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;

/// TODO: use gitignore
pub fn allowed_dir(dname: &str) -> bool {
    !dname.ends_with("/.git") && !dname.ends_with("/vendor") && !dname.ends_with("/var/cache")
}

/// finds files inside `root` w/ names that matches
/// Performance:
///   Good enough. It's not really slow and it permits the use of .into_par_iter()
pub fn f_find(root: &str, regex_match: &str) -> Vec<String> {
    let reg = Regex::new(regex_match).unwrap();
    let mut dir_stack: Vec<String> = Vec::new();
    let mut file_stack: Vec<String> = Vec::new();

    match fs::metadata(root) {
        Ok(m) if m.is_dir() => dir_stack.push(root.to_owned()),
        Ok(m) if m.is_file() && reg.is_match(root) => file_stack.push(root.to_owned()),
        Err(e) => {
            eprintln!("Could not get info for file {} ({})", root, e);
            return file_stack;
        }
        _ => return file_stack,
    };

    while let Some(dir) = dir_stack.pop() {
        let sub = match fs::read_dir(&dir) {
            Ok(s) => s.map(|d| d.unwrap().path().to_str().unwrap().to_owned()),
            Err(e) => {
                eprintln!("Could not read directory {} ({})", dir, e);
                continue;
            }
        };
        for s in sub {
            let s_meta = match fs::metadata(&s) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Could not get info for file {} ({})", dir, e);
                    continue;
                }
            };
            match () {
                _ if s_meta.is_dir() && allowed_dir(&s) => dir_stack.push(s),
                _ if s_meta.is_file() && reg.is_match(&s) => file_stack.push(s),
                _ => (),
            };
        }
    }
    file_stack
}

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
                eprintln!("Could not read {} ({})", file_name, e);
                return None;
            }
        };

        Some(FileTransformer {
            contents,
            read_ofst: 0,
        })
    }
    pub fn reader_replace(&mut self, re_start: usize, re_end: usize, replacement: &str) {
        let before = re_start + self.read_ofst;
        let after = re_end + self.read_ofst;
        self.contents = format!(
            "{}{}{}",
            &self.contents[..before],
            replacement,
            &self.contents[after..]
        );
        self.read_ofst = before + replacement.len();
    }
    pub fn reset_reader(&mut self) {
        self.read_ofst = 0;
    }
    pub fn reader(&self) -> &str {
        let clen = self.contents.len();
        match self.read_ofst < clen {
            true => &self.contents[self.read_ofst..],
            _ if clen != 0 => &self.contents[clen - 1..],
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

enum ReplacePart<'a> {
    S(&'a str),
    M(usize),
}
struct SearchReplace<'a> {
    search: Regex,
    replace: Vec<ReplacePart<'a>>,
}

fn parse_escaped<'a>(sr: (&str, &'a str)) -> SearchReplace<'a> {
    let (sear, repl) = sr;
    let search = Regex::new(&regex::escape(sear)).unwrap();
    let replace = vec![ReplacePart::S(repl)];
    SearchReplace { search, replace }
}
fn parse<'a>(sr: (&str, &'a str)) -> SearchReplace<'a> {
    let (sear, repl) = sr;
    let reg_match = Regex::new(r"\$\(([0-9]*)\)").unwrap();
    let search = Regex::new(sear).unwrap();
    let mut replace = Vec::new();
    let mut reader_ofst = 0;
    for m in reg_match.captures_iter(repl) {
        let full = m.get(0).unwrap();
        let nb = m.get(1).unwrap();
        if full.start() == 0 || repl.as_bytes()[full.start() - 1] != b'$' {
            replace.push(ReplacePart::S(&repl[reader_ofst..full.start()]));
            replace.push(ReplacePart::M(nb.as_str().parse().unwrap()));
        } else {
            replace.push(ReplacePart::S(&repl[reader_ofst..full.start() - 1]));
            replace.push(ReplacePart::S(&repl[full.start()..full.end()]));
        }
        reader_ofst = full.end();
    }
    replace.push(ReplacePart::S(&repl[reader_ofst..repl.len()]));
    SearchReplace { search, replace }
}

fn sr_file(fname: &str, search_replace: &Vec<SearchReplace>) {
    let mut ft = match FileTransformer::new(&fname) {
        Some(ft) => ft,
        None => return,
    };

    let mut modified = false;
    for sr in search_replace.iter() {
        ft.reset_reader();
        while let Some(cap) = sr.search.captures(ft.reader()) {
            let s = cap.get(0).unwrap().start();
            let a = cap.get(0).unwrap().end();
            let mut new_text = String::new();
            for part in &sr.replace {
                match part {
                    ReplacePart::S(s) => new_text.push_str(s),
                    ReplacePart::M(m) => new_text.push_str(&cap[*m]),
                }
            }
            ft.reader_replace(s, a, &new_text);
            modified = true;
        }
    }
    if modified {
        ft.write_file(&fname);
    }
}

fn sr_files(inv: Invocation) -> Result<&'static str, MV8Error> {
    let escape: bool = &*inv.args.get(1).as_string().unwrap().to_string() == "true";
    let mut search = Vec::<String>::new();
    let search_len = inv.args.get(3).as_array().unwrap().len();
    for i in 0..search_len {
        search.push(inv.args.get(3).as_array().unwrap().get::<Value>(i).unwrap().as_string().unwrap().to_string())
    }
    let mut replace = Vec::<String>::new();
    let replace_len = inv.args.get(4).as_array().unwrap().len();
    for i in 0..replace_len {
        replace.push(inv.args.get(4).as_array().unwrap().get::<Value>(i).unwrap().as_string().unwrap().to_string())
    }
    let where_: String = inv.args.get(0).as_string().unwrap().to_string();
    let glob: String = inv.args.get(2).as_string().unwrap().to_string();

    let parser = if escape { parse_escaped } else { parse };
    let mut search_replace: Vec<SearchReplace> = Vec::<SearchReplace>::new();
    for i in 0..search_len {
        search_replace.push(parser((&search[i as usize], &replace[i as usize])));
    }

    let files = f_find(&*where_, &*glob);

    files
        .into_par_iter()
        .for_each(|f| sr_file(&f, &search_replace));

    return Ok("");
}

pub fn sr_files_bridge(mv8: &MiniV8) -> Function {
    return mv8.create_function(sr_files);
}