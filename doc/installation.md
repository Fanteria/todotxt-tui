# Installation

## crates.io

You can install the application directly from [crates.io](https://crates.io/crates/todotxt-tui) with the following command:

```bash
cargo install todotxt-tui
```

## Manual

1. Clone the [repository](https://github.com/Fanteria/todotxt-tui).
2. Build the application using Rust's package manager, Cargo.

```bash
cargo build --release
```

Copy the executable from the target directory to a directory included in your system's PATH.

```bash
cp target/release/todotxt-tui /usr/local/bin/
```

## Initial setup

To set up the basic configuration, create a directory called `todotxt-tui` in your configuration folder (_the default is `$HOME/.config`_), and export the default configuration to this directory:

```bash
todo-tui --export-default-config "$HOME/.config/todotxt-tui/todotxt-tui.toml"
```

Next, open the configuration file and set the `todo_path` to the full path of your `todo.txt` file.

