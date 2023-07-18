## sql-parse

Build on top of tree-sitter (language parser) to allow for semantic searches and perform
operations of the SQL code found within Python code.

#### Usage

```
$ sql-parse --help
```

By default, the tool uses a tree-sitter query to search for the string passed to
the first parameter of `crs.execute` or `crs.executemany`.
```
$ sql-parse ts path/to/file.py
$ sql-parse ts path/to/file1.py path/to/file2.py
```

Specify a custom query file with `-t`
```
$ sql-parse ts -t path/to/treesitter/query path/to/file.py
```


<br>

Further narrow down search using regex with the `regex` subcommand.
```
$ sql-parse regex 'DECLARE @' path/to/file.py
```

<br>
Paths to search can also be piped in from stdin.

eg: using ripgrep to find all files containing `crs.execute`
```
$ rg -l -i 'crs\.execute' | sql-parse regex 'DECLARE @'
```

<br>

Find options to subcommands with
```
$ sql-parse <subcommand> --help
```
