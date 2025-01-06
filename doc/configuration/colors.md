# Colors

In Todo.txt TUI, you can customize the colors and text styles for various elements to suit your preferences. The customization options include setting foreground (`fg`) and background (`bg`) colors, as well as applying text modifiers to style the text.

**Defining Colors**

You can specify colors in the following formats:

- Named Colors: Use standard color names, e.g., `"Black"`, `"Red"`, etc.
- RGB Values: Use hexadecimal codes, e.g., `"#ff0000"` for red.
- Terminal Index: Use terminal color indices, e.g., `"9"` for bright red.

**Applying Text Modifiers**

Modifiers allow you to style text with additional visual effects. Available modifiers include:

- `Bold`: Makes the text bold.
- `Italic`: Applies italic styling.
- `Underlined`: Underlines the text.

**Example Configuration**

To configure custom colors and text modifiers for the project `+todo-tui`, update your Todo.txt TUI application's TOML configuration file as follows:

```toml
[custom_category_style."+todo-tui"]
fg = "#ff0000"      # Set foreground color to red (RGB value)
bg = "Black"        # Set background color to black (named color)
modifiers = "Italic"  # Apply italic styling
```

This configuration will style the text for `+todo-tui` with a red foreground, black background, and italicized text.

## Options

<dt><b>Flag:</b> <code>-A</code>, <code>--list-active-color</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_LIST_ACTIVE_COLOR</code></dt>
<dt><b>Conf:</b> <code>list_active_color</code></dt>
<dd>
Specifies the visual style used to highlight the currently active item in a list. This style helps you quickly identify the selected task.

**default:**
```toml
bg = "LightRed"
```
</dd>
<br>

<dt><b>Flag:</b> <code>-P</code>, <code>--pending-active-color</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PENDING_ACTIVE_COLOR</code></dt>
<dt><b>Conf:</b> <code>pending_active_color</code></dt>
<dd>
Specifies the text style for highlighting active tasks in the pending list. This overrides `list_active_color`.

</dd>
<br>

<dt><b>Flag:</b> <code>-D</code>, <code>--done-active-color</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_DONE_ACTIVE_COLOR</code></dt>
<dt><b>Conf:</b> <code>done_active_color</code></dt>
<dd>
Specifies the text style for highlighting active tasks in the completed list. This overrides `list_active_color`.
</dd>
<br>

<dt><b>Flag:</b> <code>--category-active-color</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_CATEGORY_ACTIVE_COLOR</code></dt>
<dt><b>Conf:</b> <code>category_active_color</code></dt>
<dd>
Specifies the text style for highlighting active categories. This overrides `list_active_color`.
</dd>
<br>

<dt><b>Flag:</b> <code>--projects-active-color</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PROJECTS_ACTIVE_COLOR</code></dt>
<dt><b>Conf:</b> <code>projects_active_color</code></dt>
<dd>
Specifies the text style for highlighting active projects. This overrides `category_active_color`.
</dd>
<br>

<dt><b>Flag:</b> <code>--contexts-active-color</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_CONTEXTS_ACTIVE_COLOR</code></dt>
<dt><b>Conf:</b> <code>contexts_active_color</code></dt>
<dd>
Specifies the text style for highlighting active contexts. This overrides `category_active_color`.
</dd>
<br>

<dt><b>Flag:</b> <code>--tags-active-color</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_TAGS_ACTIVE_COLOR</code></dt>
<dt><b>Conf:</b> <code>tags_active_color</code></dt>
<dd>
Specifies the text style for highlighting active tags. This overrides `category_active_color`.
</dd>
<br>
