# fedik

fedik is a TUI Clock inspired by `tty-clock` written in Rust.

## Features

- Display the current time in a terminal interface
- weekday and millisecond display

## Usage

```bash
fedik --help
fedik -swd
```

### Options
- `-s, --show-seconds`
- `-m, --ms-digits <1|2|3>`
- `-d, --show-date`
- `-w, --show-week`
- `-u, --utc`
- `-b, --bold`

### Not implemented yet
- `-c, --center`
- `-t, --hour-12`
- `-f, --format`


## Installation

```bash
cargo install --path .
```
