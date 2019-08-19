[![Build Status](https://travis-ci.org/herzrasen/pkghist.svg?branch=master)](https://travis-ci.org/herzrasen/pkghist)
[![codecov](https://codecov.io/gh/herzrasen/pkghist/branch/master/graph/badge.svg)](https://codecov.io/gh/herzrasen/pkghist)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/herzrasen/pkghist/blob/master/LICENSE)

# pkghist
`pkghist` queries your `Pacman` logfile for the local history of either single packages or your entire installation.

## About
`pkghist` parses Pacman's logfile (usually located under `/var/log/pacman.log` or specified via `--logfile /path/to/pacman.log`) and outputs version information. 
`pkghist` can list information about currently uninstalled packages using the `--removed-only` or `--with-removed` options. 

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
pkghist 0.0.1
Trace package versions from pacman's logfile

USAGE:
    pkghist [FLAGS] [OPTIONS] [filter]...

FLAGS:
    -h, --help            Prints help information
        --no-colors       Disable colored output
    -R, --removed-only    Only output packages that are currently uninstalled
    -V, --version         Prints version information
    -v, --verbose         Set the level of verbosity
    -r, --with-removed    Include packages that are currently uninstalled

OPTIONS:
    -d, --depth <depth>                    How many versions to go back in report [default: all]
    -l, --logfile <FILE>                   Specify a logfile [default: /var/log/pacman.log]
    -o, --output-format <output-format>    Select the output format [default: plain]  [possible values: json, plain]

ARGS:
    <filter>...    Filter the packages that should be searched for

```

## Usage
### List all installed packages in alphabetical order (with information about updates)
```
pkghist 
```

This will output a (possibly large) list of installed packages and their update history:
```
accountsservice
  [2019-03-03 13:06:00] Installed
  [2019-03-14 20:44:00] Upgraded
  [2019-03-26 21:49:00] Removed
  [2019-04-23 22:53:00] Installed
  [2019-04-26 22:26:00] Upgraded
acl
  [2019-03-03 10:02:00] Installed
acpi_call
  [2019-03-03 11:36:00] Installed
  [2019-03-05 21:22:00] Upgraded
  [2019-03-13 22:38:00] Upgraded
  [2019-03-15 14:37:00] Upgraded
  [2019-03-21 22:58:00] Upgraded
  [2019-03-24 22:35:00] Upgraded
  [2019-03-30 22:09:00] Upgraded
  [2019-04-05 23:13:00] Upgraded
  [2019-04-08 22:31:00] Upgraded
  [2019-04-21 01:00:00] Upgraded
  [2019-04-23 06:57:00] Upgraded
  [2019-04-30 11:41:00] Upgraded
  [2019-05-04 14:32:00] Upgraded
  [2019-05-05 20:58:00] Upgraded
  [2019-05-06 14:08:00] Upgraded
  [2019-05-15 21:16:00] Upgraded
  [2019-05-20 11:27:00] Upgraded
  [2019-05-22 14:55:00] Upgraded
  [2019-05-23 06:58:00] Upgraded
  [2019-05-28 15:36:00] Upgraded
  [2019-06-03 11:48:00] Upgraded
  [2019-06-05 22:02:00] Upgraded
  [2019-06-10 22:18:00] Upgraded
  [2019-06-13 12:45:00] Upgraded
  [2019-06-19 06:33:00] Upgraded
  [2019-06-20 23:15:00] Upgraded
  [2019-06-23 21:09:00] Upgraded
  [2019-06-26 12:48:00] Upgraded
  [2019-07-08 01:01:00] Upgraded
  [2019-07-11 22:08:00] Upgraded
  [2019-07-16 21:09:00] Upgraded
  [2019-07-25 01:16:00] Upgraded
  [2019-07-27 00:13:00] Upgraded
  [2019-07-30 21:44:00] Upgraded
  [2019-08-01 22:08:00] Upgraded
  [2019-08-06 21:24:00] Upgraded
  [2019-08-09 22:27:00] Upgraded
  [2019-08-12 07:12:00] Upgraded
  [2019-08-17 21:58:00] Upgraded
adapta-gtk-theme
  [2019-03-03 13:50:00] Installed
adobe-source-code-pro-fonts
  [2019-03-16 21:35:00] Installed
...
```