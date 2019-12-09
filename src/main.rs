#[macro_use]
extern crate clap;

mod file_finder;
mod file_transformer;

use file_finder::f_find;
use file_transformer::FileTransformer;
use itertools::zip_eq;
use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;

fn parse_escaped<'a>(sr: (&str, &'a str)) -> (Regex, &'a str) {
    (Regex::new(&regex::escape(sr.0)).unwrap(), sr.1)
}
fn parse<'a>(sr: (&str, &'a str)) -> (Regex, &'a str) {
    (Regex::new(sr.0).unwrap(), sr.1)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = clap_app!(myapp =>
        (version: "0.1")
        (author: "Arthur W. <arthur.woimbee@gmail.com>")
        (about: "rsr")
        (@arg WHERE: +takes_value +required "Where to search & replace")
        (@arg ESCAPE: -e --escape "Escape search string")
        (@arg SEARCH: -s --search ... +takes_value +required "What to search")
        (@arg REPLACE: -r --replace ... +takes_value +required "replace by what")
    )
    .get_matches();

    let escape = args.is_present("ESCAPE");
    let search = args.values_of("SEARCH").unwrap();
    let replace = args.values_of("REPLACE").unwrap();
    let where_ = args.value_of("WHERE").unwrap();

    let parser = if escape { parse_escaped } else { parse };
    let search_replace: Vec<(Regex, &str)> = zip_eq(search, replace).map(parser).collect(); // THIS SUCKS

    let files = f_find(where_, ".*");

    for fname in files {
        let mut ft = match FileTransformer::new(&fname) {
            Some(ft) => ft,
            None => continue,
        };

        let mut modified = false;
        for sr in search_replace.iter() {
            ft.reset_reader();
            while let Some(m) = sr.0.find(ft.reader()) {
                let s = m.start();
                let a = m.end();

                ft.reader_replace(s, a, sr.1);
                modified = true;
            }
        }
        if modified {
            ft.write_file(&fname);
        }
    }

    Ok(())
}
