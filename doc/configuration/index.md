# Configuration

Todo.txt TUI provides extensive customization options through a TOML configuration file located in the `todotxt-tui/todotxt-tui.toml` directory. Configuration settings can be overridden using flags or environment variables, with the following priority order: 

1. Flags  
2. Environment variables  
3. Configuration file  

For a complete list of available flags, use the help option (`--help`).

## Configuration Folder Lookup

The application checks for the configuration folder in the following order:

1. `$XDG_CONFIG_HOME`  
2. If `$XDG_CONFIG_HOME` is not set, `$HOME/.config` is used.  

The default configuration folder can be changed using the `--config-path <PATH>` flag.

## Relationship Between Flags, Environment Variables, and Configuration Keys

Flags, environment variables, and configuration keys share a consistent naming pattern:

- Flags use kebab-case (e.g., `--pending-sort`).  
- Environment variables are in uppercase, use underscores instead of hyphens, and are prefixed with `TODOTXT_TUI_` (e.g., `$TODOTXT_TUI_PENDING_SORT`).  
- Configuration keys use snake_case (e.g., `pending_sort`).  

## Value Consistency

Most values are interchangeable between flags, environment variables, and configuration keys. However, when using the configuration file, values should be written in PascalCase instead of kebab-case.

### Example

For `alphanumeric-reverse`:
- Flag: `--alphanumeric-reverse`  
- Environment variable: `$TODOTXT_TUI_ALPHANUMERIC_REVERSE`  
- Configuration key (in the TOML file): `AlphanumericReverse`
