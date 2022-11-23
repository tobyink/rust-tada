# tada

Command-line [todo.txt](https://github.com/todotxt/todo.txt) manager.

[![CI](https://github.com/tobyink/rust-tada/actions/workflows/ci.yml/badge.svg)](https://github.com/tobyink/rust-tada/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/tobyink/rust-tada/branch/master/graph/badge.svg?token=4B6I1ovnvW)](https://codecov.io/gh/tobyink/rust-tada)

## Status

Early implementation stage.

## Usage

The `tada` command operates on files called "todo.txt" and "done.txt"
in your home directory by default. These should be formatted using the
[todo.txt](https://github.com/todotxt/todo.txt) format; one task per line.
Empty lines are allowed, and lines starting with "#" are ignored as comments.

You can use environment variables `TODO_FILE` and `DONE_FILE` to point
to different files, or use command-line flags for the same. The `TODO_DIR`
environment variable also exists and affects both files.

Tada usually categorizes tasks along three main vectors:

* urgency - that is, after what date will it be "too late" to do the task
* importance - how vital is it that the task is done at all
* tshirt size - is the task small, medium, or large?

People often confuse urgency with importance. Watching a football match on TV
might be urgent because the game starts in ten minutes, but it's probably not
all that important. Filing your taxes might not be urgent, but because the
consequences of not doing it are dire, it's important. Importance is indicated
by setting a priority capital `(A)` to `(E)`. (Letters after `E` are allowed,
but will be treated as essentially equivalent.)

Due dates are indicated by including `due:YYYY-MM-DD` in the task description,
and urgency will be calculated based on how soon the due date is, or if it is
overdue.

Tshirt size is indicated by marking the task with `@S`, `@M`, or `@L`. As a
rough guide, tasks under an hour might be small, tasks under a day might be
medium, and anything else might be large. But you know better than I how big
your tasks normally are, so different thresholds may make sense for you. Tada
doesn't make any assumptions about how big `@S`, `@M`, and `@L` are in terms
or minutes, hours, or days; just that those three sizes exist. Tags like
`@XS` and `@XXL` are allowed, but will be treated as synonyms for `@S` and
`@L`.

```text
A todo list manager

Usage: tada <COMMAND>

Commands:
  add        Add a task to the todo list
  remove     Remove a task or tasks
  edit       Open your todo list in your editor
  done       Mark a task or tasks as done
  find       Search for a task
  show       Show the full todo list
  important  Show the most important tasks
  urgent     Show the most urgent tasks
  quick      Show the smallest tasks
  archive    Move completed tasks from todo.txt to done.txt
  tidy       Remove blank lines and comments from a todo list
  zen        Automatically reschedule overdue tasks
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

The shortcuts `tada i`, `tada u`, and `tada q` can be used to show important,
urgent, and quick tasks.

The shortcuts `tada +project`, `tada @context`, and `tada #linenumber`
can be used as shortcuts for finding projects by project, context, and
line number.

### tada add

```text
Add an item to the todo list

Usage: tada add [OPTIONS] [task]

Arguments:
  [task]  Task text (may use todo.txt features)

Options:
  -f, --file <FILE>       the path or URL for todo.txt
  -l, --local             look for files in local directory only
      --no-date           Don't automatically add a creation date to the task
      --max-width <COLS>  maximum width of terminal output
      --colour            coloured output
      --no-colour         plain output
  -L, --show-lines        show line numbers for tasks
      --show-created      show 'created' dates for tasks
      --show-finished     show 'finished' dates for tasks
  -h, --help              Print help information

After success, displays the added task.
```

### tada remove

```text
Remove a task or tasks

Usage: tada remove [OPTIONS] <search-term>...

Arguments:
  <search-term>...  a tag, context, line number, or string

Options:
  -f, --file <FILE>       the path or URL for todo.txt
  -l, --local             look for files in local directory only
      --max-width <COLS>  maximum width of terminal output
      --colour            coloured output
      --no-colour         plain output
  -L, --show-lines        show line numbers for tasks
      --show-created      show 'created' dates for tasks
      --show-finished     show 'finished' dates for tasks
  -y, --yes               assume 'yes' to prompts
  -n, --no                assume 'no' to prompts
  -h, --help              Print help information
```

### tada edit

```text
Open your todo list in your editor

Usage: tada edit [OPTIONS]

Options:
  -f, --file <FILE>  the path or URL for todo.txt
  -l, --local        look for files in local directory only
  -h, --help         Print help information

Ensure the EDITOR environent variable is set.
```

### tada done

```text
Mark a task or tasks as done

Usage: tada done [OPTIONS] <search-term>...

Arguments:
  <search-term>...  a tag, context, line number, or string

Options:
  -f, --file <FILE>       the path or URL for todo.txt
  -l, --local             look for files in local directory only
      --max-width <COLS>  maximum width of terminal output
      --colour            coloured output
      --no-colour         plain output
  -L, --show-lines        show line numbers for tasks
      --show-created      show 'created' dates for tasks
      --show-finished     show 'finished' dates for tasks
      --no-date           Don't automatically add a completion date to the task
  -y, --yes               assume 'yes' to prompts
  -n, --no                assume 'no' to prompts
  -h, --help              Print help information
```

### tada find

```text
Search for a task

Usage: tada find [OPTIONS] <search-term>...

Arguments:
  <search-term>...  a tag, context, line number, or string

Options:
  -f, --file <FILE>       the path or URL for todo.txt
  -l, --local             look for files in local directory only
      --max-width <COLS>  maximum width of terminal output
      --colour            coloured output
      --no-colour         plain output
  -L, --show-lines        show line numbers for tasks
      --show-created      show 'created' dates for tasks
      --show-finished     show 'finished' dates for tasks
  -s, --sort <BY>         sort by 'smart', 'urgency', 'importance' (default), 'size', 'alpha', or 'due'
  -h, --help              Print help information

Multiple search terms may be provided, which will be combined with an 'AND' operator.

Searches are case-insensitive.
```

### tada show

```text
Show the full todo list

Usage: tada show [OPTIONS]

Options:
  -f, --file <FILE>       the path or URL for todo.txt
  -l, --local             look for files in local directory only
      --max-width <COLS>  maximum width of terminal output
      --colour            coloured output
      --no-colour         plain output
  -L, --show-lines        show line numbers for tasks
      --show-created      show 'created' dates for tasks
      --show-finished     show 'finished' dates for tasks
  -s, --sort <BY>         sort by 'smart' (default), 'urgency', 'importance', 'size', 'alpha', 'due', or 'orig'
  -i, --importance        group by importance
  -u, --urgency           group by urgency
  -z, --size              group by tshirt size
  -h, --help              Print help information
```

### tada important


```text
Show the most important tasks

Usage: tada important [OPTIONS]

Options:
  -f, --file <FILE>       the path or URL for todo.txt
  -l, --local             look for files in local directory only
      --max-width <COLS>  maximum width of terminal output
      --colour            coloured output
      --no-colour         plain output
  -L, --show-lines        show line numbers for tasks
      --show-created      show 'created' dates for tasks
      --show-finished     show 'finished' dates for tasks
  -n, --number <N>        maximum number to show (default: 3)
  -s, --sort <BY>         sort by 'smart', 'urgency', 'importance' (default), 'size', 'alpha', or 'due'
  -h, --help              Print help information

Ignores tasks which are marked as already complete.
```

### tada urgent

```text
Show the most urgent tasks

Usage: tada urgent [OPTIONS]

Options:
  -f, --file <FILE>       the path or URL for todo.txt
  -l, --local             look for files in local directory only
      --max-width <COLS>  maximum width of terminal output
      --colour            coloured output
      --no-colour         plain output
  -L, --show-lines        show line numbers for tasks
      --show-created      show 'created' dates for tasks
      --show-finished     show 'finished' dates for tasks
  -n, --number <N>        maximum number to show (default: 3)
  -s, --sort <BY>         sort by 'smart', 'urgency', 'importance', 'size', 'alpha', or 'due' (default)
  -h, --help              Print help information

Ignores tasks which are marked as already complete.
```

### tada quick

```text
Show the smallest tasks

Usage: tada quick [OPTIONS]

Options:
  -f, --file <FILE>       the path or URL for todo.txt
  -l, --local             look for files in local directory only
      --max-width <COLS>  maximum width of terminal output
      --colour            coloured output
      --no-colour         plain output
  -L, --show-lines        show line numbers for tasks
      --show-created      show 'created' dates for tasks
      --show-finished     show 'finished' dates for tasks
  -n, --number <N>        maximum number to show (default: 3)
  -s, --sort <BY>         sort by 'smart', 'urgency', 'importance' (default), 'size', 'alpha', or 'due'
  -h, --help              Print help information

Ignores tasks which are marked as already complete.
```

### tada archive

```text
Move completed tasks from todo.txt to done.txt

Usage: tada archive [OPTIONS]

Options:
  -f, --file <FILE>       the path or URL for todo.txt
  -l, --local             look for files in local directory only
      --done-file <FILE>  the path or URL for done.txt
  -h, --help              Print help information
```

### tada tidy

```text
Remove blank lines and comments from a todo list

Usage: tada tidy [OPTIONS]

Options:
  -f, --file <FILE>  the path or URL for todo.txt
  -l, --local        look for files in local directory only
  -s, --sort <BY>    sort by 'smart', 'urgency', 'importance', 'size', 'alpha', 'due', or 'orig' (default)
  -h, --help         Print help information

This is the only command which will renumber tasks in your todo list.
```

### tada zen

```text
Automatically reschedule overdue tasks

Usage: tada zen [OPTIONS]

Options:
  -f, --file <FILE>  the path or URL for todo.txt
  -l, --local        look for files in local directory only
  -h, --help         Print help information

Zen will reschedule any overdue tasks on your todo list. It does not consult you
to ask for a new due date, but guesses when a sensible due date might be.
```

Exactly how zen works is subject to change, but it will aim to reschedule
tasks which are both small *and* important to be done first, then tasks which
are either small *or* important, and finally larger and less important tasks.
It will only reschedule tasks which are already overdue and not finished.

### Protocol Support

It is possible to set `TADA_FILE` or the `--file` option to an HTTP or HTTPS
URL. It performs `GET` requests to read the file and `PUT` to write to it.

The `TADA_HTTP_USER_AGENT`, `TADA_HTTP_AUTHORIZATION`, and `TADA_HTTP_FROM`
environment variables may be used to perform some very rudimentary
authentication if the server at the other end of the request is set up
right. See [php-tada-server](https://github.com/tobyink/php-tada-server)
for an example.

## Licence

This project is triple licensed under the [Apache License, version 2.0](http://www.apache.org/licenses/LICENSE-2.0), the [MIT License](http://opensource.org/licenses/MIT), and the [GNU General Public License, version 2.0](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion into this project by you, shall be triple licensed as Apache-2.0/MIT/GPL-2.0, without any additional terms or conditions.
