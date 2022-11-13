# tada

Command-line [todo.txt](https://github.com/todotxt/todo.txt) manager.

[![CI](https://github.com/tobyink/rust-tada/actions/workflows/ci.yml/badge.svg)](https://github.com/tobyink/rust-tada/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/tobyink/rust-tada/branch/master/graph/badge.svg?token=4B6I1ovnvW)](https://codecov.io/gh/tobyink/rust-tada)

## Status

Early implementation stage.

## Usage

The `tada` command operates on files called "todo.txt" and "done.txt"
in your home directory by default.

You can use environment variables `TODO_FILE` and `DONE_FILE` to point
to different files, or use command-line flags for the same. The `TODO_DIR`
environment variable also exists and affects both files.

```text
A todo list manager

Usage: tada <COMMAND>

Commands:
  show  Show the full todo list
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help information
```

### tada show

```text
Show the full todo list

Usage: tada show [OPTIONS]

Options:
  -f, --file <FILE>       the path to todo.txt
      --max-width <COLS>  maximum width of terminal output
      --colour            coloured output
      --no-colour         plain output
      --show-created      show 'created' dates for tasks
      --show-finished     show 'finished' dates for tasks
  -s, --sort <BY>         sort by 'smart' (default), 'urgency', 'importance', 'size', 'alpha', 'due', or 'orig'
  -i, --importance        group by importance
  -u, --urgency           group by urgency
  -z, --size              group by tshirt size
  -h, --help              Print help information
```

## Licence

This project is triple licensed under the [Apache License, version 2.0](http://www.apache.org/licenses/LICENSE-2.0), the [MIT License](http://opensource.org/licenses/MIT), and the [GNU General Public License, version 2.0](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion into this project by you, shall be triple licensed as Apache-2.0/MIT/GPL-2.0, without any additional terms or conditions.
