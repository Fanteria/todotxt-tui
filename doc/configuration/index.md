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

## Keybindings

Keybindings follow a specific format and can be customized through configuration files or command-line arguments. Only events mentioned in their respective configuration sections are valid.

### General Syntax

Keybindings are defined using the format:

```toml
"key_with_modifiers" = "Event"
```

- **`<key with modifiers>`**: Defines the key combination for triggering the event.
- **`Event`**: Specifies the action that will be performed when the keybinding is triggered.

### Key Rules

1. **Case Insensitivity**: Keys are case-insensitive. For example, `j` and `J` are equivalent, as are `enter` and `Enter`.
2. **Non-Character Keys**: Special keys like `enter`, `backspace`, and `escape` must be explicitly named.
3. **Special Characters**: The characters `+`, `,`, and `:` have special meanings and must be named explicitly as `plus`, `comma`, and `doubledot`, respectively.

### Modifiers

Modifiers are optional and must be separated by a `+` symbol. Supported modifiers are:

- **`shift`** (or `S`)
- **`alt`** (or `A`)
- **`ctrl`** (or `C`)

Modifiers can be written in full or abbreviated to their first letter. For example:

- `Shift+j` = `S+j` = `SHIFT+J`

It is possible to use multiple modifiers in a single keybinding.

### Examples

#### Basic Keybinding

```toml
j = "MoveDown"
```

#### Keybinding with Modifiers

```toml
"Shift+j" = "MoveDown"
"Ctrl+alt+k" = "MoveUp"
"Shift+Enter" = "Select" 
```

## Remapping Keybindings

When remapping keybindings, you must explicitly unmap the current binding using the `None` event. For example:

### Example Remapping

To remap `Shift+j = MoveDown` to `Down = MoveDown`:

```toml
"Shift+j" = "None"
Down = "MoveDown"
```

If you do not unmap the original keybinding, both keybindings will trigger the same action.

## Command-Line Configuration

Keybindings can also be configured via the command line using a compact syntax. Use the format:

```plaintext
[<binding1>:<event1>,<binding2>:<event2>,...]
```

### Example

To set keybindings directly from the command line:

```plaintext
[S+j:MoveDown,S+k:MoveUp]
```

This syntax is equivalent to setting:

```toml
"Shift+j" = "MoveDown"
"Shift+k" = "MoveUp"
```



