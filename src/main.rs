#[macro_use]
extern crate clap;

mod file_finder;
mod file_transformer;

use file_finder::f_find;
use file_transformer::FileTransformer;
use rayon::prelude::*;
use regex::Regex;
use std::error::Error;

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

fn main() -> Result<(), Box<dyn Error>> {
    let args = clap_app!(rsr =>
        (version: "0.5")
        (author: "Arthur W. <arthur.woimbee@gmail.com>")
        (about: "rsr, a tool to search & replace FAST.")
        (@arg WHERE: +takes_value +required "Where to search & replace")
        (@arg ESCAPE: -e --escape "Escape search string")
        (@arg SEARCH: -s --search ... +takes_value +required "What to search (regex syntax unless --escape)")
        (@arg REPLACE: -r --replace ... +takes_value +required "Replace by what ? (capture groups: $(N), \\n: $'\\n')")
        (@arg GLOB: -g --glob +takes_value "Kinda a glob pattern (regex syntax)")
    )
    .get_matches();

    if args.occurrences_of("SEARCH") != args.occurrences_of("REPLACE") {
        panic!("You must input 1 replacement string per search string !");
    }

    let escape = args.is_present("ESCAPE");
    let search = args.values_of("SEARCH").unwrap();
    let replace = args.values_of("REPLACE").unwrap();
    let where_ = args.value_of("WHERE").unwrap();
    let glob = args
        .value_of("GLOB")
        .unwrap_or(".*(?:(?:[^p]|p[^n]|pn[^g])|(?:[^g]|g[^i]|gi[^f]))");

    let parser = if escape { parse_escaped } else { parse };
    let search_replace: Vec<_> = search.zip(replace).map(parser).collect();

    let files = f_find(where_, glob);

    files
        .into_par_iter()
        .for_each(|f| sr_file(&f, &search_replace));
    Ok(())
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
