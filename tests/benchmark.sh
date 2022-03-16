#!/usr/bin/env bash

set -eux

cargo build --release --locked

sudo -v

# Cold cache
hyperfine -p 'sudo tee /proc/sys/vm/drop_caches <<< 3' \
	'./target/release/findpkg typo' \
	'! pkgfile -bv -- typo'

# Warm cache
hyperfine -w 3 \
	'./target/release/findpkg typo' \
	'! pkgfile -bv -- typo' \
