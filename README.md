# Again - Alias manager

Again is a simple alias manager for shell commands. It can also be used as a note
taking tool for commands you don't want to forget.

[![asciicast](https://asciinema.org/a/V56hp096fdkJJhmMrNrqtBnCd.svg)](https://asciinema.org/a/V56hp096fdkJJhmMrNrqtBnCd)

## Motivation

I wrote this because I often find useful commands I want to remembar, but since
I don't don't have a daily use for them, I keep forgetting. I do search my bash
history with Ctrl-R, but sometimes I don't remembar what to look for, sometimes
I don't use a command for so long it gets deleted.

- DO use it for long commands you don't want to commit to a git repository.
- DO NOT use it for long commands that could be useful to others. Write a script
  instead.
- DO NOT use it for aliases you'll want to use forever. Write a proper alias in
  your `~/.bashrc` instead.
- DO use it as a note taking tool for commands you rarely use but don't want to
  forget.

## Installation

Installation requires Rust and cargo:
```sh
# cargo install --git https://github.com/MatteoNardi/again
```

You may want to alias it to a shorter name:
```sh
# echo alias ag=again >> ~/.bashrc
# source ~/.bashrc
```

You can enable autocompletions with:
```sh
# cargo run -- completions bash > ~/.config/bash_completion
```

If you aliased it to something else, use:
```sh
# cargo run -- completions bash --exe ag > ~/.config/bash_completion
```

## Usage

```sh
# again --help
again 0.1.0
Matteo Nardi <matteo@matteonardi.org>
A commands alias manager

USAGE:
    again <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    completions    Generate again shell completions for your shell to stdout
    delete         Delete an alias
    edit           Edit a command in your editor
    help           Print this message or the help of the given subcommand(s)
    list           List aliases
    rename         Rename an alias
    run            Run an alias
    save           Save an alias

# again save hello echo hello world

# again ls
hello: echo hello world

# again run hello
hello: echo hello world
hello world

# cat ~/.config/ag_registry/ag_registry.toml
hello = 'echo hello world'

# again delete hello
Deleted hello:
"echo hello world"
```

## Tips & tricks

This program works great with bash history substitution. The most important
thing to know is that `!!` gets replaced with the last typed command
(See `man history` for more details)

```sh
# echo some long and complicated program
some long and complicated program

# ag save complicated !!
ag save complicated echo some long and complicated program
complicated:
  echo some long and complicated program

# ag run complicated
complicated: echo some long and complicated program
some long and complicated program
```

## Thanks

A special thanks to my employer [Exein](https://www.exein.io/) for being awesome and
granting us weekly half-day slots for OSS work on personal projects.
