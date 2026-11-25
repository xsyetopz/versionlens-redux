#!/usr/bin/perl
# Reads the LSP traffic that the zed-gui wrapper records, one framed message after another, so the
# run can pick menu entries by command and check what the server answered.
#   lsp.pl actions OUT [URI]     counts the code action responses that list VersionLens commands
#   lsp.pl pick OUT URI COMMAND  the position of COMMAND from the bottom of the latest menu for URI
#   lsp.pl lens OUT COMMAND      the position of the COMMAND lens among the lenses on its line
#   lsp.pl ran IN COMMAND        whether Zed asked the server to execute COMMAND
#   lsp.pl failed IN OUT         lists executed commands the server answered with an error
#   lsp.pl answered IN           whether Zed answered a window/showMessageRequest with a choice
use strict;
use warnings;
use JSON::PP;

sub messages {
	my ($file) = @_;
	open my $fh, '<:raw', $file or return ();
	local $/;
	my $text = <$fh> // '';
	my @out;
	while ($text =~ /Content-Length: *(\d+)\r\n(?:[^\r\n]+\r\n)*\r\n/g) {
		my $len = $1;
		my $body = substr $text, pos($text), $len;
		pos($text) += length $body;
		my $msg = eval { decode_json($body) };
		push @out, $msg if ref $msg eq 'HASH';
	}
	return @out;
}

sub command_of {
	my ($entry) = @_;
	my $command = $entry->{command};
	return ref $command eq 'HASH' ? $command->{command} // '' : $command // '';
}

sub menus {
	my ($file, $uri) = @_;
	return grep {
		my $r = $_->{result};
		ref $r eq 'ARRAY'
			&& (grep { ref $_ eq 'HASH' && ($_->{kind} // '') eq 'source.versionlens' } @$r)
			&& (!defined $uri || grep { names($_, $uri) } @$r);
	} messages($file);
}

# Zed refetches the actions of open documents in the background; a menu names its document in
# the arguments of its per-file commands.
sub names {
	my ($entry, $uri) = @_;
	my $command = $entry->{command};
	return ref $command eq 'HASH' && grep { ($_ // '') eq $uri } @{ $command->{arguments} // [] };
}

my ($mode, $file, $arg, $extra) = @ARGV;
if ($mode eq 'actions') {
	print scalar(menus($file, $arg)), "\n";
} elsif ($mode eq 'pick') {
	my ($menu) = reverse menus($file, $arg);
	my @commands = reverse map { command_of($_) } @{ $menu ? $menu->{result} : [] };
	my ($at) = grep { $commands[$_] eq $extra } 0 .. $#commands;
	defined $at or exit 1;
	print "$at\n";
} elsif ($mode eq 'lens') {
	my ($lenses) = reverse grep {
		my $r = $_->{result};
		ref $r eq 'ARRAY' && grep { ref $_ eq 'HASH' && command_of($_) eq $arg } @$r;
	} messages($file);
	$lenses or exit 1;
	my @all = @{ $lenses->{result} };
	my ($target) = grep { command_of($_) eq $arg } @all;
	my $line = $target->{range}{start}{line};
	my @row = grep { $_->{range}{start}{line} == $line } @all;
	my ($at) = grep { command_of($row[$_]) eq $arg } 0 .. $#row;
	print "$at\n";
} elsif ($mode eq 'ran') {
	exit !grep { ($_->{method} // '') eq 'workspace/executeCommand' && ($_->{params}{command} // '') eq $arg }
		messages($file);
} elsif ($mode eq 'failed') {
	my %reply = map { defined $_->{id} && !defined $_->{method} ? ($_->{id} => $_) : () } messages($arg);
	for my $request (grep { ($_->{method} // '') eq 'workspace/executeCommand' } messages($file)) {
		my $reply = $reply{ $request->{id} };
		print "$request->{params}{command}: ", ($reply ? encode_json($reply->{error}) : 'no answer'), "\n"
			if !$reply || $reply->{error};
	}
} elsif ($mode eq 'answered') {
	exit !grep {
		!defined $_->{method} && ref $_->{result} eq 'HASH' && defined $_->{result}{title}
	} messages($file);
} else {
	die "lsp.pl: unknown mode $mode\n";
}
