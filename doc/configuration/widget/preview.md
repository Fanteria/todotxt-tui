# Preview

Defines the format string used to generate the preview pane, which provides a detailed view of the selected task. You can use placeholders to include dynamic content based on task attributes and apply text styling with the new format syntax.

**Formatting Rules**

Text Colors: Enclose text in `[...]` and specify the style in parentheses. As style you can use any color definition from [Colors](../colors.md)

**Examples:**
- `[some text](Red)` sets the text to have a red foreground.
- `[some text](^Red)` sets the text to have a red background.
- `[some text](Blue Bold)` sets the text to have a bold, blue foreground.

Dynamic Variables: Insert task-specific values using $name. You can use the following variables to represent task attributes:

| Variable        | Description                                                      |
| :-------------- | :--------------------------------------------------------------- |
| $pending        | Number of pending tasks.                                         |
| $done           | Number of completed tasks.                                       |
| $subject        | Task subject.                                                    |
| $priority       | Task priority.                                                   |
| $create_date    | Task creation date.                                              |
| $finish_date    | Task finish date.                                                |
| $finished       | Task finished status (true or false).                            |
| $threshold_date | Task threshold date.                                             |
| $due_date       | Task due date.                                                   |
| $contexts       | Task contexts.                                                   |
| $projects       | Task projects.                                                   |
| $hashtags       | Task hashtags.                                                   |
| other           | Special values for custom key-value pairs in the todo.txt format |

Example format string:

```plaintext
[Pending: $pending](#ff0000) [Done: $done](^Green)
[Subject: $subject](Blue Bold)
[Priority: $priority](priority:A)
[Created: $create_date](^Yellow)
[Link: $link](^9)
```

This example configures the preview pane with styled text for pending and completed tasks, task details, and dynamic values based on attributes.

## Options

<dt><b>Flag:</b> <code>-p</code>, <code>--preview-format</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PREVIEW_FORMAT</code></dt>
<dt><b>Conf:</b> <code>preview_format</code></dt>
<dd>

Defines the format string used to generate the preview pane, which provides a detailed view of the selected task. Placeholders allow dynamic content customization based on task attributes. Format description is above.

**default:**

```plaintext
Pending: $pending Done: $done
Subject: $subject
Priority: $priority
Create date: $create_date
Link: $link"""
```

</dd>
<br>

<dt><b>Flag:</b> <code>-w</code>, <code>--wrap-preview</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_INIT_WIDGET</code></dt>
<dt><b>Conf:</b> <code>wrap_preview</code></dt>
<dd>
Determines whether the text in the preview pane should wrap to fit the available display width. When enabled, long lines are broken into multiple lines for better readability.

**default:** `true`

</dd>
<br>
