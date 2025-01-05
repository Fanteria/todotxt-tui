# Installation

## crates.io

You can install the application directly from [crates.io](https://crates.io/crates/todotxt-tui) using the following command:

```bash
cargo install todotxt-tui
```

## Manual Installation

1. Clone the repository.

2. Build the application using Rust's package manager, Cargo:

```bash
cargo build --release
```

Copy the executable from the target directory to a directory included in your system's `PATH`:

```bash
cp target/release/todotxt-tui /usr/local/bin/
```

## Initial Setup

To set up the basic configuration, create a directory named todotxt-tui in your configuration folder (_the default is `$HOME/.config`_). Then, export the default configuration to this directory:

```bash
todotxt-tui --export-default-config "$HOME/.config/todotxt-tui/todotxt-tui.toml"
```

If todotxt-tui does not find the configuration folder, it will prompt you to create one.

Next, open the configuration file and set the `todo_path` to the path of your `todo.txt` file. Environment variables or `~` will be expanded if the path is in UTF-8 format.

## Autocomplete

You can generate an autocomplete script for Bash and source it:

```bash
todotxt-tui --export-autocomplete ./autocomplete-todotxt-tui.sh
source ./autocomplete-todotxt-tui.sh
```

Alternatively, if you prefer on-the-fly autocompletion, you can add this to your `.bashrc`:

```bash
source <(todotxt-tui --export-autocomplete /dev/stdout)
```
