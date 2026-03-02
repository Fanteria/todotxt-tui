# Widget

<dt><b>Flag:</b> <code>-T</code>, <code>--tasks-keybind</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_TASKS_KEYBIND</code></dt>
<dt><b>Conf:</b> <code>tasks_keybind</code></dt>
<dd>

Defines the [keybindings](../index.md#keybindings) used for interacting with task items within the application. These bindings allow you to quickly navigate, edit, and perform actions on tasks.

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

Specifies the [keybindings](../index.md#keybindings) for managing and navigating categories such as projects and contexts. These categories help you organize tasks effectively.

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

<dt><b>Flag:</b> <code>--pending-widget-name</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PENDING_WIDGET_NAME</code></dt>
<dt><b>Conf:</b> <code>pending_widget_name</code></dt>
<dd>

The title label displayed on the pending tasks widget border.

**default:** `"list"`
</dd>
<br>

<dt><b>Flag:</b> <code>--done-widget-name</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_DONE_WIDGET_NAME</code></dt>
<dt><b>Conf:</b> <code>done_widget_name</code></dt>
<dd>

The title label displayed on the done tasks widget border.

**default:** `"done"`
</dd>
<br>

<dt><b>Flag:</b> <code>--project-widget-name</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PROJECT_WIDGET_NAME</code></dt>
<dt><b>Conf:</b> <code>project_widget_name</code></dt>
<dd>

The title label displayed on the projects category widget border.

**default:** `"project"`
</dd>
<br>

<dt><b>Flag:</b> <code>--context-widget-name</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_CONTEXT_WIDGET_NAME</code></dt>
<dt><b>Conf:</b> <code>context_widget_name</code></dt>
<dd>

The title label displayed on the contexts category widget border.

**default:** `"context"`
</dd>
<br>

<dt><b>Flag:</b> <code>--hashtag-widget-name</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_HASHTAG_WIDGET_NAME</code></dt>
<dt><b>Conf:</b> <code>hashtag_widget_name</code></dt>
<dd>

The title label displayed on the hashtags category widget border.

**default:** `"hashtag"`
</dd>
<br>

<dt><b>Flag:</b> <code>--preview-widget-name</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PREVIEW_WIDGET_NAME</code></dt>
<dt><b>Conf:</b> <code>preview_widget_name</code></dt>
<dd>

The title label displayed on the preview widget border.

**default:** `"preview"`
</dd>
<br>

<dt><b>Flag:</b> <code>--pending-live-preview-widget-name</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PENDING_LIVE_PREVIEW_WIDGET_NAME</code></dt>
<dt><b>Conf:</b> <code>pending_live_preview_widget_name</code></dt>
<dd>

The title label displayed on the pending live preview widget border.

**default:** `"pending live preview"`
</dd>
<br>

<dt><b>Flag:</b> <code>--done-live-preview-widget-name</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_DONE_LIVE_PREVIEW_WIDGET_NAME</code></dt>
<dt><b>Conf:</b> <code>done_live_preview_widget_name</code></dt>
<dd>

The title label displayed on the done live preview widget border.

**default:** `"done live preview"`
</dd>
<br>
