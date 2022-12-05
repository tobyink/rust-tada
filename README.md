# tada

Command-line [todo.txt](https://github.com/todotxt/todo.txt) manager.

[![Latest version](https://img.shields.io/crates/v/tada.svg)](https://crates.io/crates/tada) [![CI](https://github.com/tobyink/rust-tada/actions/workflows/ci.yml/badge.svg)](https://github.com/tobyink/rust-tada/actions/workflows/ci.yml) [![codecov](https://codecov.io/gh/tobyink/rust-tada/branch/master/graph/badge.svg?token=4B6I1ovnvW)](https://codecov.io/gh/tobyink/rust-tada) [![Documentation](https://docs.rs/tada/badge.svg)](https://docs.rs/tada) [![Licence](https://img.shields.io/crates/l/tada.svg)](https://github.com/tobyink/rust-tada#licence)

## Status

Early development, but usable.

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
overdue. It is also possible to set start dates on tasks using
`start:YYYY-MM-DD`. Tasks will be shown greyed out until that date.
(All dates are just dates, not datetimes.)

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
  pull       Reschedule a task or tasks to be done today (or another
                 date)
  done       Mark a task or tasks as done
  find       Search for a task
  show       Show the full todo list
  important  Show the most important tasks
  urgent     Show the most urgent tasks
  quick      Show the smallest tasks
  archive    Move completed tasks from todo.txt to done.txt
  tidy       Remove blank lines and comments from a todo list
  zen        Automatically reschedule overdue tasks
  path       Prints the full path to your todo list
  help       Print this message or the help of the given
                 subcommand(s)

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
Add a task to the todo list

Usage: tada add [OPTIONS] [task]

Arguments:
  [task]  Task text (may use todo.txt features)

Options:
  -f, --file <FILE>       The path or URL for todo.txt
  -l, --local             Look for files in local directory only
      --no-date           Don't automatically add a creation date to the
                          task
      --no-fixup          Don't try to fix task syntax
      --quiet             Quieter output
  -T, --today             Include a due date of today
  -S, --soon              Include a due date of overmorrow
  -W, --next-week         Include a due date the end of next week
  -M, --next-month        Include a due date the end of next month
      --colour            Coloured output
      --no-colour         Plain output
      --max-width <COLS>  Maximum width of terminal output
  -L, --show-lines        Show line numbers for tasks
      --show-created      Show 'created' dates for tasks
      --show-finished     Show 'finished' dates for tasks
  -h, --help              Print help information

After success, displays the added task.
```

### tada remove

```text
Remove a task or tasks

Usage: tada remove [OPTIONS] <search-term>...

Arguments:
  <search-term>...  A tag, context, line number, or string

Options:
  -f, --file <FILE>       The path or URL for todo.txt
  -l, --local             Look for files in local directory only
      --colour            Coloured output
      --no-colour         Plain output
      --max-width <COLS>  Maximum width of terminal output
  -L, --show-lines        Show line numbers for tasks
      --show-created      Show 'created' dates for tasks
      --show-finished     Show 'finished' dates for tasks
  -y, --yes               Assume 'yes' to prompts
  -n, --no                Assume 'no' to prompts
  -h, --help              Print help information
```

### tada edit

```text
Open your todo list in your editor

Usage: tada edit [OPTIONS]

Options:
  -f, --file <FILE>  The path or URL for todo.txt
  -l, --local        Look for files in local directory only
  -h, --help         Print help information

Ensure the EDITOR environent variable is set.
```

### tada pull

```text
Reschedule a task or tasks to be done today (or another date)

Usage: tada pull [OPTIONS] <search-term>...

Arguments:
  <search-term>...  A tag, context, line number, or string

Options:
  -f, --file <FILE>       The path or URL for todo.txt
  -l, --local             Look for files in local directory only
      --colour            Coloured output
      --no-colour         Plain output
      --max-width <COLS>  Maximum width of terminal output
  -L, --show-lines        Show line numbers for tasks
      --show-created      Show 'created' dates for tasks
      --show-finished     Show 'finished' dates for tasks
  -T, --today             Set a due date of today (default)
  -S, --soon              Set a due date of overmorrow
  -W, --next-week         Set a due date the end of next week
  -M, --next-month        Set a due date the end of next month
  -y, --yes               Assume 'yes' to prompts
  -n, --no                Assume 'no' to prompts
  -h, --help              Print help information

If a task has a start date, that will be set to today.
```

### tada done

```text
Mark a task or tasks as done

Usage: tada done [OPTIONS] <search-term>...

Arguments:
  <search-term>...  A tag, context, line number, or string

Options:
  -f, --file <FILE>       The path or URL for todo.txt
  -l, --local             Look for files in local directory only
      --colour            Coloured output
      --no-colour         Plain output
      --max-width <COLS>  Maximum width of terminal output
  -L, --show-lines        Show line numbers for tasks
      --show-created      Show 'created' dates for tasks
      --show-finished     Show 'finished' dates for tasks
      --no-date           Don't automatically add a completion date to
                          the task
  -y, --yes               Assume 'yes' to prompts
  -n, --no                Assume 'no' to prompts
  -h, --help              Print help information
```

### tada find

```text
Search for a task

Usage: tada find [OPTIONS] <search-term>...

Arguments:
  <search-term>...  A tag, context, line number, or string

Options:
  -f, --file <FILE>       The path or URL for todo.txt
  -l, --local             Look for files in local directory only
      --colour            Coloured output
      --no-colour         Plain output
      --max-width <COLS>  Maximum width of terminal output
  -L, --show-lines        Show line numbers for tasks
      --show-created      Show 'created' dates for tasks
      --show-finished     Show 'finished' dates for tasks
  -s, --sort <BY>         Sort by 'smart', 'urgency', 'importance',
                          'size', 'alpha', or 'due' (default: smart)
  -h, --help              Print help information

Multiple search terms may be provided, which will be combined with an
'AND' operator.

Searches are case-insensitive.
```

### tada show

```text
Show the full todo list

Usage: tada show [OPTIONS]

Options:
  -f, --file <FILE>       The path or URL for todo.txt
  -l, --local             Look for files in local directory only
      --colour            Coloured output
      --no-colour         Plain output
      --max-width <COLS>  Maximum width of terminal output
  -L, --show-lines        Show line numbers for tasks
      --show-created      Show 'created' dates for tasks
      --show-finished     Show 'finished' dates for tasks
  -s, --sort <BY>         Sort by 'smart' (default), 'urgency',
                          'importance', 'size', 'alpha', 'due', or
                          'orig'
  -i, --importance        Group by importance
  -u, --urgency           Group by urgency
  -z, --size              Group by tshirt size
  -h, --help              Print help information
```

### tada important

```text
Show the most important tasks

Usage: tada important [OPTIONS]

Options:
  -f, --file <FILE>       The path or URL for todo.txt
  -l, --local             Look for files in local directory only
      --colour            Coloured output
      --no-colour         Plain output
      --max-width <COLS>  Maximum width of terminal output
  -L, --show-lines        Show line numbers for tasks
      --show-created      Show 'created' dates for tasks
      --show-finished     Show 'finished' dates for tasks
  -n, --number <N>        Maximum number to show (default: 3)
  -s, --sort <BY>         Sort by 'smart', 'urgency', 'importance',
                          'size', 'alpha', or 'due' (default:
                          importance)
  -h, --help              Print help information

Ignores tasks which are marked as already complete or have a start date
in the future.
```

### tada urgent

```text
Show the most urgent tasks

Usage: tada urgent [OPTIONS]

Options:
  -f, --file <FILE>       The path or URL for todo.txt
  -l, --local             Look for files in local directory only
      --colour            Coloured output
      --no-colour         Plain output
      --max-width <COLS>  Maximum width of terminal output
  -L, --show-lines        Show line numbers for tasks
      --show-created      Show 'created' dates for tasks
      --show-finished     Show 'finished' dates for tasks
  -n, --number <N>        Maximum number to show (default: 3)
  -s, --sort <BY>         Sort by 'smart', 'urgency', 'importance',
                          'size', 'alpha', or 'due' (default: urgency)
  -h, --help              Print help information

Ignores tasks which are marked as already complete or have a start date
in the future.
```

### tada quick

```text
Show the smallest tasks

Usage: tada quick [OPTIONS]

Options:
  -f, --file <FILE>       The path or URL for todo.txt
  -l, --local             Look for files in local directory only
      --colour            Coloured output
      --no-colour         Plain output
      --max-width <COLS>  Maximum width of terminal output
  -L, --show-lines        Show line numbers for tasks
      --show-created      Show 'created' dates for tasks
      --show-finished     Show 'finished' dates for tasks
  -n, --number <N>        Maximum number to show (default: 3)
  -s, --sort <BY>         Sort by 'smart', 'urgency', 'importance',
                          'size', 'alpha', or 'due' (default: size)
  -h, --help              Print help information

Ignores tasks which are marked as already complete or have a start date
in the future.
```

### tada archive

```text
Move completed tasks from todo.txt to done.txt

Usage: tada archive [OPTIONS]

Options:
  -f, --file <FILE>       The path or URL for todo.txt
  -l, --local             Look for files in local directory only
      --done-file <FILE>  The path or URL for done.txt
      --colour            Coloured output
      --no-colour         Plain output
  -h, --help              Print help information
```

### tada tidy

```text
Remove blank lines and comments from a todo list

Usage: tada tidy [OPTIONS]

Options:
  -f, --file <FILE>  The path or URL for todo.txt
  -l, --local        Look for files in local directory only
  -s, --sort <BY>    Sort by 'smart', 'urgency', 'importance', 'size',
                     'alpha', 'due', or 'orig' (default)
  -h, --help         Print help information

This is the only command which will renumber tasks in your todo list.
```

### tada zen

```text
Automatically reschedule overdue tasks

Usage: tada zen [OPTIONS]

Options:
  -f, --file <FILE>  The path or URL for todo.txt
  -l, --local        Look for files in local directory only
      --colour       Coloured output
      --no-colour    Plain output
  -h, --help         Print help information

Zen will reschedule any overdue tasks on your todo list. It does not
consult you to ask for a new due date, but guesses when a sensible due
date might be.
```

Exactly how zen works is subject to change, but it will aim to reschedule
tasks which are both small *and* important to be done first, then tasks which
are either small *or* important, and finally larger and less important tasks.
It will only reschedule tasks which are already overdue and not finished.

### tada path

```text
Prints the full path to your todo list

Usage: tada path [OPTIONS]

Options:
  -f, --file <FILE>  The path or URL for todo.txt
  -l, --local        Look for files in local directory only
  -h, --help         Print help information

This allows things like:

  /path/to/some/editor `tada path`
```

### tada help

```text
Print this message or the help of the given subcommand(s)

Usage: tada help [COMMAND]...

Arguments:
  [COMMAND]...  Print help for the subcommand(s)
```

### Recurring Tasks

`tada` does not have any explicit support for recurring tasks. However it
should be simple to set up a cron job to add tasks to your list on schedule.
For example, to add an urgent task to your todo list every Friday morning:

```text
1  0  *  *  5  tada add --today 'Take out the trash @home'
```

### Protocol Support

It is possible to set `TADA_FILE` or the `--file` option to an HTTP or HTTPS
URL. It performs `GET` requests to read the file and `PUT` to write to it.

The `TADA_HTTP_USER_AGENT`, `TADA_HTTP_AUTHORIZATION`, and `TADA_HTTP_FROM`
environment variables may be used to perform some very rudimentary
authentication if the server at the other end of the request is set up
right. See [php-tada-server](https://github.com/tobyink/php-tada-server)
for an example.

### File Format

As mentioned above, todo files are expected to be in the
[todo.txt](https://github.com/todotxt/todo.txt) format. Lines consisting
of just whitespace are ignored. Lines beginning with `#` are treated as
comments and ignored.

`tada` recognizes the following special tags in descriptions:

#### Context Tags

- The `@S`, `@M`, and `@L` contexts are used to indicate whether a task is small, medium, or large.
- If a task has context `@work` or `@school`, it will avoid being automatically rescheduled onto Saturdays or Sundays.

#### Key-Value Tags

- `due:YYYY-MM-DD` sets a due date for a task.
- `start:YYYY-MM-DD` sets a start date for a task.

## Licence

This project is triple licensed under the [Apache License, version 2.0](http://www.apache.org/licenses/LICENSE-2.0), the [MIT License](http://opensource.org/licenses/MIT), and the [GNU General Public License, version 2.0](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion into this project by you, shall be triple licensed as Apache-2.0/MIT/GPL-2.0, without any additional terms or conditions.
