# findpkg

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

| Hook    | Cold cache | Warm cache |
| ------- | ---------: | ---------: |
| findpkg | 40ms       | 1.4ms      |
| pkgfile | 1874ms     | 440ms      |

## Installation

Install [findpkg](https://aur.archlinux.org/packages/findpkg) from the [AUR](https://wiki.archlinux.org/title/Arch_User_Repository)

```sh
yay -S findpkg
```

Create/update the database

```sh
sudo pkgfile --update
```

Enable automatic database updates (Optional)

```sh
systemctl enable pkgfile.timer
```

### Bash

Add the following to `~/.bashrc`

```bash
command_not_found_handle() {
	findpkg "$1"
}
```

### fish

Run:

```fish
function fish_command_not_found
    findpkg $argv[1]
end

funcsave fish_command_not_found
```

### Zsh

Add the following to `~/.zshrc`

```zsh
command_not_found_handler() {
	findpkg "$1"
}
```
