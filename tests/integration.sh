#!/bin/sh

set -eux

cargo build --release --locked
bin=./target/release/fast-command-not-found

$bin -u
stat /var/lib/fast-command-not-found/database

test "$($bin ls)" = "\
core/coreutils     	/usr/bin/ls
community/9base    	/opt/plan9/bin/ls
community/plan9port	/usr/lib/plan9/bin/ls"

test "$($bin unknown)" = "Command not found: unknown"
