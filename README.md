# fast-command-not-found

A fast *command not found* handler for `pacman`. It suggests which package to
install when you try to run a command from a package that isn't installed

```
$ tree
tree may be found in the following packages:
  extra/tree    /usr/bin/tree
```

The output is more or less the same as [`pkgfile`'s hook](https://wiki.archlinux.org/title/Pkgfile#Command_not_found)
but is substantially faster. `pkgfile` can take several seconds to complete,
which is easily confused for a successful command invocation

| Hook                   | Cold cache | Warm cache |
| ---------------------- | ---------: | ---------: |
| fast-command-not-found | 40ms       | 1.4ms      |
| pkgfile                | 1874ms     | 440ms      |
