#!/bin/sh

set -eux

cargo build --release --locked
findpkg=./target/release/findpkg

$findpkg --update

test "$($findpkg ls)" = "ls may be found in the following packages:
  core/coreutils     	/usr/bin/ls
  community/9base    	/opt/plan9/bin/ls
  community/plan9port	/usr/lib/plan9/bin/ls"

test "$($findpkg unknown)" = "Command not found: unknown"

stat /var/lib/findpkg/database
