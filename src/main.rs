mod file_finder;
mod file_transformer;
mod modifiers;

use file_finder::FileWalker;
use file_transformer::FileTransformer;
use modifiers::{get_modifier, DynFnPtr};

use clap::{arg, command};
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use regex::Regex;
use std::borrow::Cow;
use std::error::Error;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Where to search & replace
    #[arg()]
    r#where: PathBuf,

    /// What to search (regex syntax unless --escape)
    #[arg(short, long)]
    search: Vec<String>,

    /// Replace by what ? (capture groups: $(N), \n: $'\n')
    #[arg(short, long)]
    replace: Vec<String>,

    ///Glob file paths (regex syntax)
    #[arg(short, long, default_value = ".*")]
    glob: String,

    /// Escape search string
    #[arg(short, long)]
    escape: bool,

    /// Print to stdout, do not modify files
    #[arg(short, long)]
    dry_run: bool,
}

struct Match {
    id: usize,
    transform: Option<DynFnPtr>,
}

impl std::fmt::Debug for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(id: {}, has_transform: {})",
            self.id,
            self.transform.is_some()
        )
    }
}

#[derive(Debug)]
enum ReplacePart<'a> {
    Str(&'a str),
    Match(Match),
}
#[derive(Debug)]
struct SearchReplace<'a> {
    search: Regex,
    replace: Vec<ReplacePart<'a>>,
}

fn parse_escaped<'a>(sr: (&str, &'a str)) -> SearchReplace<'a> {
    let (sear, repl) = sr;
    let search = Regex::new(&regex::escape(sear)).unwrap();
    let replace = vec![ReplacePart::Str(repl)];
    SearchReplace { search, replace }
}

fn parse<'a>(sr: (&str, &'a str)) -> SearchReplace<'a> {
    let reg_match = Regex::new(r"\$\(([0-9]*)([A-Z]+)?\)").unwrap();
    let (raw_search, raw_replace) = sr;
    let search = Regex::new(raw_search).unwrap();
    let mut replace = Vec::new();
    let mut read_ofset = 0;
    for m in reg_match.captures_iter(raw_replace) {
        let full = m.get(0).unwrap();
        let nb = m.get(1).unwrap();
        // if is not escaped
        if full.start() == 0 || raw_replace.as_bytes()[full.start() - 1] != b'$' {
            let id = nb.as_str().parse().unwrap();
            let transform: Option<DynFnPtr> = match m.get(2) {
                Some(c) => match get_modifier(c.as_str()) {
                    Some(m) => Some(m),
                    None => panic!("Unrecognised modifier: {}", c.as_str()),
                },
                None => None,
            };
            replace.push(ReplacePart::Str(&raw_replace[read_ofset..full.start()]));
            replace.push(ReplacePart::Match(Match { id, transform }));
        } else {
            replace.push(ReplacePart::Str(&raw_replace[read_ofset..full.end()]));
        }
        read_ofset = full.end();
    }
    replace.push(ReplacePart::Str(
        &raw_replace[read_ofset..raw_replace.len()],
    ));
    SearchReplace { search, replace }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();

    if args.search.len() != args.replace.len() {
        panic!("You must input 1 replacement string per search string !");
    }

    // Parse search & replace strings
    let parser = if args.escape { parse_escaped } else { parse };

    let search_replace: Vec<_> = args
        .search
        .iter()
        .zip(args.replace.iter())
        .map(|(s, r)| parser((s, r)))
        .collect();

    log::debug!("search_replace: {:?}", search_replace);

    // Raw sauce
    let ff = FileWalker::new(args.r#where, &args.glob);
    ff.par_bridge()
        .for_each(|f| file_search_replace(&f, &search_replace, args.dry_run));

    Ok(())
}

/// Search & Replace in one file
fn file_search_replace(f: &std::path::Path, search_replace: &[SearchReplace], dry_run: bool) {
    let mut ft = match FileTransformer::new(f) {
        Some(ft) => ft,
        None => return,
    };

    for sr in search_replace.iter() {
        ft.reset_reader();
        while let Some(cap) = sr.search.captures(ft.get_reader()) {
            let (start, end) = {
                let whole_cap = cap.get(0).unwrap();
                (whole_cap.start(), whole_cap.end())
            };
            ft.reader_push(start);
            ft.reader_skip(end - start);

            for part in &sr.replace {
                match part {
                    ReplacePart::Str(stri) => ft.push(Cow::from(*stri)),
                    ReplacePart::Match(m) => match m.transform {
                        Some(t) => ft.push(t(cap.get(m.id).unwrap().as_str())),
                        None => ft.push(Cow::from(cap.get(m.id).unwrap().as_str())),
                    },
                }
            }
        }
    }
    if dry_run {
        ft.print();
    } else {
        ft.commit();
    }
}
