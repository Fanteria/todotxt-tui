# Todo.txt TUI

Todo.txt TUI is a highly customizable terminal-based application for managing your todo tasks. It follows the todo.txt format and offers a wide range of configuration options to suit your needs. Please note that the application may have some bugs as it's still under development, so your feedback and bug reports are greatly appreciated.

[Preview.webm](https://github.com/Fanteria/todo-tui/assets/28980012/11ab70e3-482c-4994-ac88-198953b08e39)

## Installation

### crates.io

You can install the application directly from [crates.io](https://crates.io/crates/todotxt-tui) with the following command:

```bash
cargo install todotxt-tui
```

### Manual

1. Clone the [repository](https://github.com/Fanteria/todotxt-tui).
2. Build the application using Rust's package manager, Cargo.

```bash
cargo build --release
```

Copy the executable from the target directory to a directory included in your system's PATH.

```bash
cp target/release/todotxt-tui /usr/local/bin/
```

### Initial setup

To set up the basic configuration, create a directory called `todotxt-tui` in your configuration folder (_the default is `$HOME/.config`_), and export the default configuration to this directory:

```bash
todo-tui --export-default-config "$HOME/.config/todotxt-tui/todotxt-tui.toml"
```

Next, open the configuration file and set the `todo_path` to the full path of your `todo.txt` file.

## Basic Usage

Todo.txt TUI provides a straightforward and customizable interface for managing your tasks. The following keybindings and actions are available for basic usage, and please note that these actions can be configured according to your preferences in the configuration file:

- `j`: Move down in the list.
- `k`: Move up in the list.
- `g`: Go to the first item in the list.
- `G`: Go to the last item in the list.
- `Enter`: Select an item.
- `U`: Swap the selected item up.
- `D`: Swap the selected item down.
- `x`: Remove the selected item.
- `d`: Move a task between the pending and done lists.
- `I`: Input a new task.
- `E`: Edit the selected item.
- `J`: Move to the widget below the current one.
- `K`: Move to the widget above the current one.
- `H`: Move to the widget on the left.
- `L`: Move to the widget on the right.
- `q`: Quit the application.
- `Backspace`: Filter categories from the pending and done lists.
- `S`: Save tasks to the file.
- `u`: Update tasks from the file.
- `/`: Search within the current list.
- `n`: Jump to the next search result.
- `N`: Jump to the previous search result.
- `h`: Clear the search term for the current list.

## Configuration

Todo.txt TUI allows extensive customization through a TOML configuration file located in the `todotxt-tui/todotxt-tui.toml` directory. You can also use flags or environment variables to override configuration settings, following this priority order: Configuration file < Environment variables < Flags.

### Color Settings

In Todo.txt TUI, you can customize the colors and text styling for various elements. You have the flexibility to set foreground (`fg`) and background (`bg`) colors, as well as apply text modifiers for styling. Colors can be defined using color names, RGB values, or terminal index.

You can apply text modifiers to change the style of text within Todo.txt TUI. Available text modifiers include:

- `Bold`: Apply bold styling to the text.
- `Italic`: Apply italic styling to the text.
- `Underlined`: Apply underlined styling to the text.

Here's an example of how to configure custom color and text modifiers for project `todo-tui` in your Todo.txt TUI application's TOML configuration:

```toml
[custom_category_style."+todo-tui"]
fg = "#ff0000"  # Set foreground color to red using RGB values
bg = "Black"      # Set background color to black
modifiers = "Italic"  # Apply italic styling
```

### Sorting Options

You can specify how tasks are sorted using the `pending_sort` and `done_sort` options. The available sorting options are:

- `None`: No specific sorting; tasks appear in the order they were added.
- `Reverse`: Reverse the order of tasks.
- `Priority`: Sort tasks by priority.
- `Alphanumeric`: Sort tasks in alphanumeric order.
- `AlphanumericReverse`: Sort tasks in reverse alphanumeric order.

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

Feel free to adjust these settings to create a Todo.txt TUI interface that suits your workflow and preferences.

## Feedback and Bug Reporting

As this application is still in development, your feedback is greatly appreciated. If you encounter any issues or have suggestions for improvement, please open an issue on the GitHub repository to assist me in making Todo.txt TUI better.
