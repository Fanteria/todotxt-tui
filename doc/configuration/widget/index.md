# Widget

<dt><b>Flag:</b> <code>-T</code>, <code>--tasks-keybind</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_TASKS_KEYBIND</code></dt>
<dt><b>Conf:</b> <code>tasks_keybind</code></dt>
<dd>
Defines the keybindings used for interacting with task items within the application. These bindings allow you to quickly navigate, edit, and perform actions on tasks.

**default:**
```toml
d = "MoveItem"
x = "RemoveItem"
D = "SwapDownItem"
n = "NextSearch"
U = "SwapUpItem"
Enter = "Select"
N = "PrevSearch"
```
</dd>
<br>

<dt><b>Flag:</b> <code>-C</code>, <code>--category-keybind</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_CATEGORY_KEYBIND</code></dt>
<dt><b>Conf:</b> <code>category_keybind</code></dt>
<dd>
Specifies the keybindings for managing and navigating categories such as projects and contexts. These categories help you organize tasks effectively.

**default:**
```toml
Enter = "Select"
n = "NextSearch"
N = "PrevSearch"
Backspace = "Remove"
```
</dd>
<br>

<dt><b>Flag:</b> <code>-i</code>, <code>--border-type</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_BORDER_TYPE</code></dt>
<dt><b>Conf:</b> <code>border_type</code></dt>
<dd>
Defines the style of borders used in the application's user interface. Options include:

**possible values (flag, env):** `plain`, `rounded`, `double`, `thick`
**possible values (conf):** `Plain`, `Rounded`, `Double`, `Thick`
**default:** `Rounded`
</dd>
<br>
