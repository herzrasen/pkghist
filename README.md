[![Build Status](https://travis-ci.org/herzrasen/pkghist.svg?branch=master)](https://travis-ci.org/herzrasen/pkghist)
[![codecov](https://codecov.io/gh/herzrasen/pkghist/branch/master/graph/badge.svg)](https://codecov.io/gh/herzrasen/pkghist)
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
```
cargo install --path .
```

This will build `pkghist` and install it into your cargo bin directory (usually `~/.cargo/bin`).

## Help
```bash
pkghist 0.3.0
Trace package versions from pacman's logfile

USAGE:
    pkghist [FLAGS] [OPTIONS] [filter]...

FLAGS:
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
                   pattern to match (e.g. '^linux$' only matches the package 'linux'
```

## Usage
### List all installed packages ordered by install/upgrade date
```
pkghist 
```

### List the last `n` installed / upgraded packages
```
pkghist --last <n>
```

### Limit the number of versions per package
```
pkghist --limit <n>
```

### Search for a package by exact name
```
pkghist '^name$'
```

This uses regex syntax to describe the pattern to search for.

#### Example
```
pkghist '^zsh$'
```
This return only the package `zsh` and not for example `zsh-syntax-highlighting`.

### Search for all packages containing some string
```
pkghist string
```

#### Example
```
pkghist zsh
```

This returns the package `zsh` as well as for example `zsh-syntax-highlighting`.

### List the package names of all removed packages
```
pkghist --no-details --removed-only
```
