# Commands

- [Typable commands](#typable-commands)
  - [Command mode syntax](#command-mode-syntax)
    - [Quoting](#quoting)
    - [Expansions](#expansions)
    - [Flags](#flags)
    - [Exceptions](#exceptions)
  - [Built-ins](#built-ins)
- [Static commands](#static-commands)

## Typable commands

Typable commands are used from command mode and may take arguments. Command mode can be activated by pressing `:`.

### Command mode syntax

Command mode has rules for parsing the command line to evaluate quotes and expansions and split the line into positional arguments and flags. Most commands use these rules but some commands have custom parsing rules (see [Exceptions](#exceptions) below).

#### Quoting

By default command arguments are split on tabs and space characters. `:open README.md CHANGELOG.md` for example should open two files, `README.md` and `CHANGELOG.md`. Arguments that contain spaces can be surrounded in single quotes (`'`) or backticks (`` ` ``) to prevent the space from separating the argument, like `:open 'a b.txt'`.

Double quotes may be used the same way, but double quotes _expand_ their inner content. `:echo "%{cursor_line}"` for example may print `1` since the variable expansion within is expanded. `:echo '%{cursor_line}'` prints `%{cursor_line}` literally though. Content within single quotes or backticks is interpreted as-is.

On Unix systems the backslash character may be used to escape certain characters depending on where it is used. Within an argument which isn't surround in quotes, the backslash can be used to escape the space or tab characters: `:open a\ b.txt` is equivalent to `:open 'a b.txt'`. The backslash may also be used to escape quote characters (`'`, `` ` ``, `"`) or the percent token (`%`) when used at the beginning of an argument. `:echo \%%sh{foo}` for example prints `%sh{foo}` instead of invoking a `foo` shell command and `:echo \"quote` prints `"quote`. The backslash character is treated literally in any other situation on Unix systems and always on Windows: `:echo \n` always prints `\n`.

#### Expansions

Expansions are patterns that Helix recognizes and replaces within the command line. Helix recognizes anything starting with a percent token (`%`) as an expansion, for example `%sh{echo hi!}`.

Expansions take the form `%[<kind>]<open><contents><close>`. In `%sh{echo hi!}`, for example, the kind is `sh` - the shell expansion - and the contents are "echo hi!", with `{` and `}` acting as opening and closing delimiters. The following open/close characters are recognized as expansion delimiter pairs: `(`/`)`, `[`/`]`, `{`/`}` and `<`/`>`. Any grapheme cluster can be used as both open and close delimiters instead however: `%{cursor_line}` is equivalent to `%|cursor_line|`, `%"cursor_line"` and even `%🏴‍☠️cursor_line🏴‍☠️`.

When no `<kind>` is provided, Helix will expand a **variable**. For example `%{cursor_line}` can be used as an argument to provide the currently focused document's primary selection cursor line as an argument. `:echo %{cursor_line}` for instance may print `1` to the statusline.

The following variables are supported:

| Name | Description |
|---   |---          |
| `cursor_line` | The line number of the primary cursor in the currently focused document, starting at 1. |
| `cursor_column` | The column number of the primary cursor in the currently focused document, starting at 1. This is counted as the number of grapheme clusters from the start of the line rather than bytes or codepoints. |
| `buffer_name` | The relative path of the currently focused document. `[scratch]` is expanded instead for scratch buffers. |
| `line_ending` | A string containing the line ending of the currently focused document. For example on Unix systems this is usually a line-feed character (`\n`) but on Windows systems this may be a carriage-return plus a line-feed (`\r\n`). The line ending kind of the currently focused document can be inspected with the `:line-ending` command. |

Aside from editor variables, the following expansions may be used:

* Unicode `%u{..}`. The contents may contain up to six hexadecimal numbers corresponding to a Unicode codepoint value. For example `:echo %u{25CF}` prints `●` to the statusline.
* Shell `%sh{..}`. The contents are passed to the configured shell command. For example `:echo %sh{echo "20 * 5" | bc}` may print `100` on the statusline on when using a shell with `echo` and the `bc` calculator installed. Shell expansions are evaluated recursively. `%sh{echo '%{buffer_name}:%{cursor_line}'}` for example executes a command like `echo 'README.md:1'`: the variables within the `%sh{..}` expansion are evaluated before executing the shell command.

As mentioned above, double quotes can be used to surround arguments with spaces but also support expansions within the quoted content unlike singe quotes or backticks. For example `:echo "circle: %u{25CF}"` prints `circle: ●` to the statusline while `:echo 'circle: %u{25CF}'` prints `circle: %u{25CF}`.

Note that expansions are only evaluated once the Enter key is pressed in command mode.

#### Flags

Command flags are optional switches that can be used to alter the behavior of a command. For example the `:sort` command accepts an optional `--reverse` (or `-r` for short) flag which, if present, causes the sort command to reverse the sorting direction. Typing the `-` character shows completions for the current command's flags, if any.

The `--` flag specifies the end of flags. All arguments after `--` are treated as positional arguments: `:open -- -a.txt` opens a file called `-a.txt`.

#### Exceptions

The following commands support expansions but otherwise pass the given argument directly to the shell program without interpreting quotes:

* `:insert-output`
* `:append-output`
* `:pipe`
* `:pipe-to`
* `:run-shell-command`

For example executing `:sh echo "%{buffer_name}:%{cursor_column}"` would pass text like `echo "README.md:1"` as an argument to the shell program: the expansions are evaluated but not the quotes.

The `:set-option` and `:toggle-option` commands use regular parsing for the first argument - the config option name - and parse the rest depending on the config option's type. `:set-option` interprets the second argument as a string for string config options and parses everything else as JSON.

`:toggle-option`'s behavior depends on the JSON type of the config option supplied as the first argument:

* Booleans: only the config option name should be provided. For example `:toggle-option auto-format` will flip the `auto-format` option.
* Strings: the rest of the command line is parsed with regular quoting rules. For example `:toggle-option indent-heuristic hybrid tree-sitter simple` cycles through "hybrid", "tree-sitter" and "simple" values on each invocation of the command.
* Numbers, arrays and objects: the rest of the command line is parsed as a stream of JSON values. For example `:toggle-option rulers [81] [51, 73]` cycles through `[81]` and `[51, 73]`.

When providing multiple values to `:toggle-option` there should be no duplicates. `:toggle-option indent-heuristic hybrid simple tree-sitter simple` for example would only toggle between "hybrid" and "tree-sitter" values.

### Built-ins

The built-in typable commands are:

{{#include ./generated/typable-cmd.md}}

## Static Commands

Static commands take no arguments and can be bound to keys. Static commands can also be executed from the command picker (`<space>?`). The built-in static commands are:

{{#include ./generated/static-cmd.md}}
