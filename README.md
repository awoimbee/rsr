# RSR

## Why

Because search & replace tools suck and are slow on big repositories. This just works, *fast*.
Be careful, you might find this tool to be too fast when you (accidentaly) run it on `/` (or `C:\` for the weirdos).

## Alternatives

[fastmod](https://github.com/facebookincubator/fastmod) and it's ancestor [codemod](https://github.com/facebook/codemod) from the people at Facebook.

## How to use

It's just a simple search & replace tool.
It's optimized to make lots of changes at once, use this at your advantage.

`./rsr <file/folder> [-e] [-g] --search "str1" "str2" "str3" --replace "repl1" "repl2" "repl3"`

- `-s --search`: list of search strings
- `-r --search`: list or replacement strings (repl1 replaces str1, repl2 str2, ...)
  The syntax for capture groups is `$(ID)`, where `ID` is a number.
  There are some modifiers like `U` (uppercase) and `L` (lowercase), to use them: `$(0U)` or `$(3L)` or ...
- `-e --escape`: escape search string (also 'escapes' replacement string) -> TL:DR: disables regex
- `-g --glob`: Regex string to filter file path/name -> exemple: '.*\\.php'
