#!/bin/sh

set -eux

cargo build --release --locked
bin=./target/release/fast-command-not-found

$bin -u

test "$($bin ls)" = "ls may be found in the following packages:
  core/coreutils     	/usr/bin/ls
  community/9base    	/opt/plan9/bin/ls
  community/plan9port	/usr/lib/plan9/bin/ls"

test "$($bin unknown)" = "Command not found: unknown"

stat /var/lib/fast-command-not-found/database
