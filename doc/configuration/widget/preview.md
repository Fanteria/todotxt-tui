# Preview

Defines the format string used to generate the preview pane, which provides a detailed view of the selected task. You can use placeholders to include dynamic content based on task attributes and apply text styling with the new format syntax.

**Formatting Rules**

Text Colors: Enclose text in `[...]` and specify the style in parentheses. As style you can use any color definition from [Colors](../colors.md). Additionally, you can use `priority` (optionally as `priority:A` for a particular priority), `custom_category`, `projects`, `contexts`, `hashtags`, `category`, or `skip_projects`, `skip_contexts`, `skip_hashtags` to remove projects, contexts, or hashtags from the content.

**Examples:**

- `[some text](Red)` sets the text to have a red foreground.
- `[some text](^Red)` sets the text to have a red background.
- `[some text](Blue, Bold)` sets the text to have a bold, blue foreground.
- `[some text +project](Green, skip_projects)` sets the text to have a green foreground and removes `+project` from it.

Dynamic Variables: Insert task-specific values using $name. You can use the following variables to represent task attributes:

| Variable        | Description                                                       |
| :-------------- | :---------------------------------------------------------------- |
| $pending        | The number of pending tasks.                                      |
| $done           | The number of completed tasks.                                    |
| $subject        | The subject of the task.                                          |
| $priority       | The task's priority.                                              |
| $create_date    | The creation date of the task.                                    |
| $finish_date    | The finish date of the task.                                      |
| $finished       | Indicates whether the task is finished (true or false).           |
| $threshold_date | The threshold date of the task.                                   |
| $due_date       | The due date of the task.                                         |
| $contexts       | The contexts associated with the task.                            |
| $projects       | The projects associated with the task.                            |
| $hashtags       | The hashtags associated with the task.                            |
| other           | Special values for custom key-value pairs in the todo.txt format. |

Variable names are case insensitive and the multi-word ones are also accepted in the camel case
form, so `$due_date` and `$DueDate` are equivalent. A name that matches none of the variables above
is looked up among the task's custom key-value pairs, so `$link` renders the value of `link:` tag.

A variable name ends on the first whitespace, newline or one of `$`, `{`, `}`, `[`, `]`, `(`, `)`.
Use the braced form `${name}` when the variable is directly followed by another character, for
example `${due_date}!`.

It is also possible to use `!` to enforce category colors.

Example format string:

```plaintext
[Pending: $pending](#ff0000) [Done: $done](^Green)
[Subject: $subject](! priority)
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
