# Configuration

Todo.txt TUI allows extensive customization through a TOML configuration file located in the `todotxt-tui/todotxt-tui.toml` directory. You can also use flags or environment variables to override configuration settings, following this priority order: Configuration file < Environment variables < Flags.

## Color Settings

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

## Sorting Options

You can specify how tasks are sorted using the `pending_sort` and `done_sort` options. The available sorting options are:

- `None`: No specific sorting; tasks appear in the order they were added.
- `Reverse`: Reverse the order of tasks.
- `Priority`: Sort tasks by priority.
- `Alphanumeric`: Sort tasks in alphanumeric order.
- `AlphanumericReverse`: Sort tasks in reverse alphanumeric order.

## Preview Format

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

## Custom Layout

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
