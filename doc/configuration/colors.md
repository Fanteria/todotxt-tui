# Colors

**Work in progress...**

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

