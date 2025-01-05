# UI

<dt><b>Flag:</b> <code>-i</code>, <code>--init-widget</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_INIT_WIDGET</code></dt>
<dt><b>Conf:</b> <code>init_widget</code></dt>
<dd>
The widget that will be active when the application starts.  

- **Possible values (flag, env):** `list`, `done`, `project`, `context`, `hashtag`, `preview`  
- **Possible values (config):** `List`, `Done`, `Project`, `Context`, `Hashtag`, `Preview`  
- **Default:** `List` 
</dd>
<br>

<dt><b>Flag:</b> <code>-t</code>, <code>--window-title</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_WINDOW_TITLE</code></dt>
<dt><b>Conf:</b> <code>window_title</code></dt>
<dd>
The title of the window when `todotxt-tui` is opened.  

- **Default:** `ToDo TUI`  
</dd>
<br>

<dt><b>Flag:</b> <code>-W</code>, <code>--window-keybinds</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_WINDOW_KEYBINDS</code></dt>
<dt><b>Conf:</b> <code>window_keybinds</code></dt>
<dd>
Defines the keybinds for window actions.  

**Default:**  
```plaintext
I  = "InsertMode"
L  = "MoveRight"
q  = "Quit"
K  = "MoveUp"
"/" = "SearchMode"
S  = "Save"
u  = "Load"
H  = "MoveLeft"
J  = "MoveDown"
E  = "EditMode"
```
</dd>
<br>

<dt><b>Flag:</b> <code>-R</code>, <code>--list-refresh-rate</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_LIST_REFRESH_RATE</code></dt>
<dt><b>Conf:</b> <code>list_refresh_rate</code></dt>
<dd>
Specifies the refresh rate for the UI when no keys are pressed. This is particularly useful if the to-do list is modified by a different program.

- **Default value:** 5 seconds
- The configuration can also specify nanoseconds for finer granularity.  

Configuration Example:
```toml
[list_refresh_rate]
secs = 5
nanos = 0
```
</dd>
<br>

<dt><b>Flag:</b> <code>-S</code>, <code>--save-state-path</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_SAVE_STATE_PATH</code></dt>
<dt><b>Conf:</b> <code>save_state_path</code></dt>
<dd>
The path to save the application's state (currently unused).
</dd>
<br>

<dt><b>Flag:</b> <code>-l</code>, <code>--layout</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_LAYOUT</code></dt>
<dt><b>Conf:</b> <code>layout</code></dt>
<dd>
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
</dd>
<br>
