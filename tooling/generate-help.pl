#!perl

use strict;
use warnings;

my %extra_help = (
	MAIN => <<'HELP',
The shortcuts `tada i`, `tada u`, and `tada q` can be used to show important,
urgent, and quick tasks.

The shortcuts `tada +project`, `tada @context`, and `tada #linenumber`
can be used as shortcuts for finding projects by project, context, and
line number.
HELP
	zen => <<'HELP',
Exactly how zen works is subject to change, but it will aim to reschedule
tasks which are both small *and* important to be done first, then tasks which
are either small *or* important, and finally larger and less important tasks.
It will only reschedule tasks which are already overdue and not finished.
HELP
);

my ( @commands, %help );
my $main_help = qx(cargo run help);

my $seen_commands = 0;
for ( split /\n/, $main_help ) {
	if ( /^Commands:/ ) {
		++$seen_commands;
		next;
	}
	next unless $seen_commands;
	
	if ( /^\s+([a-z]+)/ ) {
		push @commands, ( my $command = $1 );
		$help{$command} = qx(cargo run help $command);
	}
}

print <<"HELP";
### tada

```text
$main_help
```

$extra_help{MAIN}
HELP

print <<"HELP" for @commands;
### tada $_

```text
$help{$_}
```
@{[ defined $extra_help{$_} ? ("\n".$extra_help{$_}) : "" ]}
HELP
