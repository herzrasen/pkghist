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
    -r, --with-removed    Include packages that are currently uninstalled

OPTIONS:
        --first <n>                        Output the first 'n' pacman events
        --last <n>                         Output the last 'n' pacman events
    -L, --limit <limit>                    How many versions to go back in report
    -l, --logfile <FILE>                   Specify a logfile [default: /var/log/pacman.log]
    -o, --output-format <output-format>    Select the output format [default: plain]  [possible values: json, plain]

ARGS:
    <filter>...    Filter the packages that should be searched for
```

## Usage
### List all installed packages in ordered by install/upgrade date
```
pkghist 
```

This will output a (possibly large) list of installed packages and their update history:
```
acl
  [2019-03-03 10:02:00] Installed
    2.2.53-1
attr
  [2019-03-03 10:02:00] Installed
    2.4.48-1
autoconf
  [2019-03-03 10:02:00] Installed
    2.69-5
automake
  [2019-03-03 10:02:00] Installed
    1.16.1-1
ca-certificates
  [2019-03-03 10:02:00] Installed
    20181109-1
...
```

### List the last `n` installed / upgraded packages
```
pkghist --last <n>
```

#### Example
```
pkghist --last 2
```

This returns the most recently installed / upgraded packages:

```bash
diff-so-fancy
  [2019-08-27 06:55:00] Installed
    1.2.6-1
  [2019-08-30 21:46:00] Upgraded
    1.2.7-1
electron4
  [2019-06-29 22:33:00] Installed
    4.2.5-1
  [2019-07-02 22:44:00] Upgraded
    4.2.6-1
  [2019-07-20 21:42:00] Upgraded
    4.2.8-1
  [2019-08-12 07:12:00] Upgraded
    4.2.8-2
  [2019-08-30 21:46:00] Upgraded
    4.2.10-1
```
