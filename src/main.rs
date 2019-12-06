#[macro_use]
extern crate clap;

mod file_transformer;

use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;
use file_transformer::FileTransformer;

fn main() -> Result<(), Box<dyn Error>> {
    let args = clap_app!(myapp =>
        (version: "0.1")
        (author: "Arthur W. <arthur.woimbee@gmail.com>")
        (about: "rsr")
        (@arg SEARCH: +takes_value +required "What to search")
        (@arg REPLACE: +takes_value +required "replace by what")
        (@arg WHERE: +takes_value +required "Where to search & replace")
    ).get_matches();


    let what = args.value_of("SEARCH").unwrap();
    let replacement = args.value_of("REPLACE").unwrap();
    let search_where = args.value_of("WHERE").unwrap();

    let rg = Regex::new(what).unwrap();
    for file in WalkDir::new(search_where).into_iter().filter_map(|e| e.ok()) {
        if !file.metadata().unwrap().is_file() {
            continue;
        }
        let fname = file.path().to_str().unwrap();

        let mut ft = match FileTransformer::new(fname) {
            Some(ft) => ft,
            None => continue,
        };

        let mut modified = false;
        while let Some(m) = rg.find(ft.reader()) {
            let s = m.start();
            let a = m.end();
            ft.reader_replace(s, a, replacement);
            modified = true;
        }
        if modified {
            ft.write_file(fname);
        }
    }

    Ok(())
}
