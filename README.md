## sql-parse

Build on top of [tree-sitter](https://github.com/tree-sitter/tree-sitter) to allow for semantic searches and perform
operations on SQL code found within Python code.

#### Installation

The [release page](https://github.com/jamestrew/sql-parse/releases) includes precompiled statically linked binaries for Linux.

#### Usage

```
$ sql-parse --help
```

By default, the tool uses a tree-sitter query to search for the string passed to
the first parameter of `crs.execute` or `crs.executemany`.

```
$ sql-parse ts path/to/file.py
$ sql-parse ts path/to/file1.py path/to/file2.py
$ sql-parse ts path/to/directory/
```

Specify a custom query file with `-t`

```
$ sql-parse ts -t path/to/treesitter/query path/to/file.py
```

Some alternative queries can be found in the `queries` directory.

<br>

Further narrow the down search using regex with the `regex` subcommand.

```
$ sql-parse regex 'DECLARE @' path/to/file.py
```

<br>
Paths to search can also be piped in from stdin.

eg: using grep to find all files containing `crs.execute`

```
$ grep -rl 'crs\.execute' | sql-parse regex 'DECLARE @'
```

<br>

Find options to subcommands with

```
$ sql-parse <subcommand> --help
```

<br>

![image](https://github.com/jamestrew/sql-parse/assets/66286082/daa66764-0fc7-4c75-888b-b675208a5d54)
