# ToDo TUI

ToDo TUI is a highly customizable terminal-based application for managing your todo tasks. It follows the todo.txt format and offers a wide range of configuration options to suit your needs. Please note that the application may have some bugs as it's still under development, so your feedback and bug reports are greatly appreciated.

## Installation

1. Clone the repository or download the latest release.
2. Build the application using Rust's package manager, Cargo.

```bash
cargo build --release
```

Copy the executable from the target directory to a directory included in your system's PATH.

```bash
cp target/release/todo-tui /usr/local/bin/
```

## Configuration

ToDo TUI uses a TOML configuration file located at `~/.config/todo-tui.toml` for customization.

<details>
  <summary>Example config file</summary>
    
Cofig file with default values. And description for every setting.

```toml
# The active color for selected items
# You can set the color by name ("Blue"), by RGB values ([255, 0, 0]), or by index in the terminal (fg.Index = 5).
active_color = "Red"

# The initial widget to be displayed
init_widget = "List"

# The window title
window_title = "ToDo tui"

# The path to your todo.txt file
todo_path = "/home/jirka/todo.txt"

# Wrap long lines in the preview
wrap_preview = true

# Log file path
log_file = "log.log"

# Log format (uses placeholders)
log_format = "{d} [{h({l})}] {M}: {m}{n}"

# Log level (e.g., INFO, DEBUG)
log_level = "INFO"

# Enable file watcher for auto-reloading
file_watcher = true

# Indentation level for lists
list_shift = 4

# Sorting option for pending tasks
pending_sort = "Priority"

# Sorting option for completed tasks
done_sort = "None"

# Preview format (uses placeholders)
preview_format = """
Pending: {n}   Done: {N}
Subject: {s}
Priority: {p}
Create date: {c}
"""

# Layout configuration
layout = """
[
    Direction: Horizontal,
    Size: 50%,
    [
        List: 50%,
        Preview,
    ],
    [ Direction: Vertical,
      Done,
      [ 
        Contexts,
        Projects,
      ],
    ],
]
"""

# Priority-specific colors
[priority_colors.B]
fg = "Yellow"

[priority_colors.A]
fg = "Red"

[priority_colors.C]
fg = "Blue"

# Background color for categories
[category_color]
bg = "Blue"

# Background color for the active list item
[list_active_color]
bg = "LightRed"

# Background color for active pending tasks
[pending_active_color]

# Background color for active completed tasks
[done_active_color]

# Autosave duration (in seconds)
[autosave_duration]
secs = 900
nanos = 0

# List refresh rate (in seconds)
[list_refresh_rate]
secs = 5
nanos = 0

# Task keybindings
[[tasks_keybind.events]]
key = "Enter"
event = "Select"

[[tasks_keybind.events]]
event = "SwapDownItem"
key.Char = "D"

[[tasks_keybind.events]]
event = "SwapUpItem"
key.Char = "U"

[[tasks_keybind.events]]
event = "MoveItem"
key.Char = "d"

[[tasks_keybind.events]]
event = "RemoveItem"
key.Char = "x"

# Category keybindings
[[category_keybind.events]]
key = "Enter"
event = "Select"

# List keybindings
[[list_keybind.events]]
event = "ListLast"
key.Char = "G"

[[list_keybind.events]]
event = "ListFirst"
key.Char = "g"

[[list_keybind.events]]
event = "ListDown"
key.Char = "j"

[[list_keybind.events]]
event = "ListUp"
key.Char = "k"

# Window keybindings
[[window_keybind.events]]
event = "Quit"
key.Char = "q"

# Category style
[category_style]
fg = "DarkGray"

# Projects style
[projects_style]

# Contexts style
[contexts_style]

# Hashtags style
[hashtags_style]

# Custom category style for "todo-tui"
[custom_category_style."+todo-tui"]
fg = "LightBlue"
```

</details>

## Feedback and Bug Reporting

As this application is still in development, your feedback is invaluable. If you encounter any issues or have suggestions for improvement, please open an issue on the GitHub repository to help me make ToDo TUI better.
