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


## using with mini-v8

$ sudo gem install libv8
$ export V8_PATH=/Library/Ruby/Gems/2.6.0/gems/libv8-7.3.492.27.1-universal-darwin-19/vendor/v8
$ cargo run -- --api-key={your-api-key} [--env=dev] program

## (hold) using with v8 + Docker

$ docker-compose run uniflow-rust-client bash
$ cargo run

## (hold) using with v8

Compile V8 (Mac) : From https://v8.dev/docs/build

### install Xcode and accept its license agreement

$ (cd /usr/local/lib & git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git chromium_depot_tools)
$ export PATH=$PATH:/usr/local/lib/chromium_depot_tools >> ~/.zprofile

$ (cd /usr/local/lib & git clone https://chromium.googlesource.com/v8/v8.git)
$ cd /usr/local/lib/v8
$ fetch v8
$ cd v8
$ git pull
$ git checkout -t tags/7.1.1
$ gclient sync
$ gn gen out.gn/x64.release --args='is_debug=false target_cpu="x64" v8_target_cpu="x64" v8_use_snapshot=false'
$ ninja -C out.gn/x64.release

$ export V8_SOURCE=/usr/local/lib/v8/v8
$ export V8_LIBS=/usr/local/lib/v8/v8/out.gn/x64.release/obj
$ cargo run -- --api-key={your-api-key} [--env=dev] program