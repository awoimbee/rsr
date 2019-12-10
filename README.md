# Why rsr ?
Because search & replace tools suck and are slow on big repositories.  
This just works, *fast*.  
**MADE FOR MULTIPLE REPLACEMENTS IN LOTS OF FILES**  
**MAY BREAK YOUR SYSTEM, BE CAREFUL** (don't be an idiot, don't run this on `/`)  
## How to use
`./rsr <file/folder> [-e] [-g] --search "str1" "str2" "str3" --replace "repl1" "repl2" "repl3"`
- `-s --search`: list of search strings
- `-r --search`: list or replacement strings (repl1 replaces str1, repl2 str2, ...)
- `-e --escape`: escape search string (also 'escapes' replacement string)
- `-g --glob`: Regex string to filter file path/name

```
➜  rsr git:(master) ✗ ./target/release/rsr -h
rsr 0.5
Arthur W. <arthur.woimbee@gmail.com>
rsr, a tool to search & replace FAST.

USAGE:
    rsr [FLAGS] [OPTIONS] <WHERE> --replace <REPLACE>... --search <SEARCH>...

FLAGS:
    -e, --escape     Escape search string
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -g, --glob <GLOB>             Kinda a glob pattern (regex syntax)
    -r, --replace <REPLACE>...    Replace by what ? (capture groups: $(N), \n: $'\n')
    -s, --search <SEARCH>...      What to search (regex syntax unless --escape)

ARGS:
    <WHERE>    Where to search & replace

```
