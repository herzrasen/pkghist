[![Build Status](https://travis-ci.org/herzrasen/pkghist.svg?branch=master)](https://travis-ci.org/herzrasen/pkghist)
[![codecov](https://codecov.io/gh/herzrasen/pkghist/branch/master/graph/badge.svg)](https://codecov.io/gh/herzrasen/pkghist)
[![pkghist](https://img.shields.io/aur/version/pkghist.svg?label=pkghist)](https://aur.archlinux.org/packages/pkghist/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/herzrasen/pkghist/blob/master/LICENSE)

# pkghist
`pkghist` queries your `Pacman` logfile for the local history of either single packages or your entire installation.

## About
`pkghist` parses Pacman's logfile (usually located under `/var/log/pacman.log` or specified via `--logfile /path/to/pacman.log`) and outputs version information. 
`pkghist` can list information about currently uninstalled packages using the `--removed-only` or `--with-removed` options. 

## Install
If you are on Arch, either install `pkghist` using AUR or by building it using makepkg

```bash
git clone https://aur.archlinux.org/pkghist.git
cd pkghist
makepkg -si
```
## Build
To build `pkghist` from source you need a `Rust` installation including it's build tool `Cargo`. 
Install it either using pacman or [follow the official install guide](https://www.rust-lang.org/tools/install).

Once `Rust` and `Cargo` are up and running, simply use:
```bash
cargo install --path .
```

This will build `pkghist` and install it into your cargo bin directory (usually `~/.cargo/bin`).

## Help
```bash
pkghist 0.5.2
Trace package versions from pacman's logfile

USAGE:
    pkghist [FLAGS] [OPTIONS] [filter]...

FLAGS:
    -x, --exclude         If set, every filter result will be excluded.
    -h, --help            Prints help information
        --no-colors       Disable colored output
        --no-details      Only output the package names
    -R, --removed-only    Only output packages that are currently uninstalled
    -V, --version         Prints version information
    -r, --with-removed    Include packages that are currently uninstalled

OPTIONS:
    -a, --after <date>                     Only consider events that occurred after 'date' [Format: "YYYY-MM-DD HH:MM"]
        --first <n>                        Output the first 'n' pacman events
        --last <n>                         Output the last 'n' pacman events
    -L, --limit <limit>                    How many versions to go back in report. [limit > 0]
    -l, --logfile <FILE>                   Specify a logfile [default: /var/log/pacman.log]
    -o, --output-format <output-format>    Select the output format [default: plain]  [possible values: json, plain,
                                           compact]

ARGS:
    <filter>...    Filter the packages that should be searched for. Use regular expressions to specify the exact
                   pattern to match (e.g. '^linux$' only matches the package 'linux')
```

## Usage
### List all installed packages ordered by install/upgrade date
```bash
pkghist 
```

### List the last `n` installed / upgraded packages
```bash
pkghist --last <n>
```

### Limit the number of versions per package
```bash
pkghist --limit <n>
```

### Search for a package by exact name
```bash
pkghist '^name$'
```

This uses regex syntax to describe the pattern to search for.

#### Example
```bash
pkghist '^zsh$'
```
This return only the package `zsh` and not for example `zsh-syntax-highlighting`.

### Search for all packages containing some string
```bash
pkghist somestring
```

#### Example
```bash
pkghist zsh
```
This returns the package `zsh` as well as for example `zsh-syntax-highlighting`.

### Excluding packages
```bash
pkghist --exclude somestring
```

#### Example
```bash
pkghist --exclude '^[a-e]'
```
This excludes all packages starting with the letters a to e.

### List the package names of all removed packages
```bash
pkghist --no-details --removed-only
```

## Regex examples
This is a little collection of useful regexes that can be used for filtering.

| Regex          | Explanation                                              |
|----------------|----------------------------------------------------------|
| `'^package$'`  | Matches only packages named 'package'                    |
| `'package'`    | Matches any package containing 'package'                 |  
| `'^[a-x]'`     | Matches any package starting with the letters 'a' to 'x' |
| `'[a-x]'`      | Matches any package containing the letter 'a' to 'x'     |
| `'[^a-x]'`     | Matches any package NOT containing the letters 'a' to 'x'| 
| `'[[:digit:]]'`| Matches any package containing a digit                   |

Sometimes using `--exclude` is easier than trying to create an exclusion regex. 

## Using pkghist's output as input for pacman
You can use the result of a `pkghist` query as input for pacman.

To create an output in the matching format, use the `--no-colors` and the `--no-details' options in your query. 

The following command installs all packages that have been removed after 2019-10-01 12:00. 
```bash
sudo pacman -S $(pkghist --no-details --no-colors --removed-only --after "2019-10-01 12:00")                                             
```

The following command removes all packages that have been installed after 2019-10-02 12:00. 
```bash
sudo pacman -R $(pkghist --no-details --no-colors --after "2019-10-02 12:00")                                                            
```

## Shell completions
`pkghist` creates completion scripts for `bash`, `fish` and `zsh`.
They are created at build time using the great [clap crate](https://github.com/clap-rs/clap). 
When installing using `makepkg` (e.g. using the AUR), they are put into the appropriate location.
When installing manually, you may copy them from [the completions directory](./completions) into the appropriate location.

Note: When using zsh, enable loading of additional completions by adding the following line to your `.zshrc`
```bash
autoload -U compinit && compinit
```
