# List

<dt><b>Flag:</b> <code>-s</code>, <code>--list-shift</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_LIST_SHIFT</code></dt>
<dt><b>Conf:</b> <code>list_shift</code></dt>
<dd>
Determines the number of lines displayed above and below the currently active item in a list when scrolling. This helps maintain context around the active task during navigation.

**default:** `4`
</dd>
<br>

<dt><b>Flag:</b> <code>-L</code>, <code>--list-keybind</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_LIST_KEYBIND</code></dt>
<dt><b>Conf:</b> <code>list_keybind</code></dt>
<dd>
Configures the keybindings for interacting with task lists. These bindings enable efficient scrolling, selection, and manipulation of list items.

**default:**
```toml
G = "ListLast"
h = "CleanSearch"
k = "ListUp"
g = "ListFirst"
j = "ListDown"
```
</dd>
<br>

<dt><b>Flag:</b> <code>--pending-format</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PENDING_FORMAT</code></dt>
<dt><b>Conf:</b> <code>pending_format</code></dt>
<dd>

The format string used to render pending tasks in the list. Format is same as for [preview](./preview.md).

**default:** `[$subject](! priority)`
</dd>
<br>

<dt><b>Flag:</b> <code>--done-format</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_DONE_FORMAT</code></dt>
<dt><b>Conf:</b> <code>done_format</code></dt>
<dd>

The format string used to render completed tasks in the list. Format is same as for [preview](./preview.md).

**default:** `[$subject](! priority)`
</dd>
<br>

