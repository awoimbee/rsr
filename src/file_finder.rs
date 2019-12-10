use regex::Regex;
use std::fs;

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
