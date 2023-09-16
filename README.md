# ToDo TUI

ToDo TUI is a highly customizable terminal-based application for managing your todo tasks. It follows the todo.txt format and offers a wide range of configuration options to suit your needs. Please note that the application may have some bugs as it's still under development, so your feedback and bug reports are greatly appreciated.

## Installation

Please note that this ToDo TUI application is intended for personal use and is not published on Rust's docs.rs or crates.io. Therefore, it must be installed manually.

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

In ToDo TUI, you can customize various settings to tailor the application to your preferences. 
ToDo TUI uses a TOML configuration file located at `~/.config/todo-tui.toml` for customization.
Here's an overview of some of the key settings:

### Color Settings

In ToDo TUI, you can customize the colors and text styling for various elements. You have the flexibility to set foreground (`fg`) and background (`bg`) colors, as well as apply text modifiers for styling. Colors can be defined using color names, RGB values, or terminal index.

You can apply text modifiers to change the style of text within ToDo TUI. Available text modifiers include:

- Bold: Apply bold styling to the text.
- Italic: Apply italic styling to the text.
- Underlined: Apply underlined styling to the text.

Here's an example of how to configure custom color and text modifiers for project `todo-tui` in your ToDo TUI application's TOML configuration:

```toml
[custom_category_style."+todo-tui"]
fg = [255, 0, 0]  # Set foreground color to red using RGB values
bg = "Black"      # Set background color to black
modifiers = "Italic"  # Apply italic styling
```

### Sorting Options

You can specify how tasks are sorted using the `pending_sort` and `done_sort` options. The available sorting options are:

- None: No specific sorting; tasks appear in the order they were added.
- Reverse: Reverse the order of tasks.
- Priority: Sort tasks by priority.
- Alphanumeric: Sort tasks in alphanumeric order.
- AlphanumericReverse: Sort tasks in reverse alphanumeric order.

### Preview Format

The `preview_format` setting allows you to define the format for the task preview. You can use placeholders enclosed in `{}` to display task information. Here are the available placeholders and their corresponding values:

- `{n}`: Number of pending tasks.
- `{N}`: Number of completed tasks.
- `{s}`: Task subject.
- `{p}`: Task priority.
- `{c}`: Task creation date.
- `{f}`: Task finish date.
- `{F}`: Task finished status (true or false).
- `{t}`: Task threshold date.
- `{d}`: Task due date.
- `{C}`: Task contexts.
- `{P}`: Task projects.
- `{H}`: Task hashtags.

### Custom Layout

The layout setting allows you to define a custom layout for the application using blocks `[]`. You can specify the orientation of the blocks as either `Direction: Vertical` or `Direction: Horizontal`, along with the size of each block as a percentage or value. Within these blocks, you can include various widgets, such as:

- `List`: The main list of tasks.
- `Preview`: The task preview section.
- `Done`: The list of completed tasks.
- `Projects`: The list of projects.
- `Contexts`: The list of contexts.
- `Hashtags`: The list of hashtags.

Here's an example of a custom layout configuration:

```
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
```

This example creates a layout with a horizontal split, where the list takes up 50% of the width, and the preview occupies the remaining space. On the right side, there's a vertical split with the list of completed tasks, contexts, and projects.

Feel free to adjust these settings to create a ToDo TUI interface that suits your workflow and preferences.

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
    
# The path to your archive.txt file
# archive_path =

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
pending_sort = "None"

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
