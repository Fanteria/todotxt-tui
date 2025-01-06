# Styles

<dt><b>Flag:</b> <code>--active-color</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_ACTIVE_COLOR</code></dt>
<dt><b>Conf:</b> <code>active_color</code></dt>
<dd>
Sets the color used to indicate the active window or element within the application. This provides a visual cue for the currently focused UI component.

**default:** `Red`
</dd>
<br>

<dt><b>Flag:</b> <code>--priority-style</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PRIORITY_STYLE</code></dt>
<dt><b>Conf:</b> <code>priority_style</code></dt>
<dd>
Defines the styles applied to tasks based on their priority levels.

**default:**
```toml
A.fg = "Red"
B.fg = "Yellow"
C.fg = "Blue"
```
</dd>
<br>

<dt><b>Flag:</b> <code>--projects-style</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_PROJECTS_STYLE</code></dt>
<dt><b>Conf:</b> <code>projects_style</code></dt>
<dd>
Specifies the text style for displaying projects within the task list. Projects are identified by their `+ProjectName` syntax.
</dd>
<br>

<dt><b>Flag:</b> <code>--contexts-style</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_CONTEXTS_STYLE</code></dt>
<dt><b>Conf:</b> <code>contexts_style</code></dt>
<dd>
Sets the text style for displaying contexts in the task list. Contexts are identified by their `@ContextName` syntax.
</dd>
<br>

<dt><b>Flag:</b> <code>--hashtags-style</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_HASHTAGS_STYLE</code></dt>
<dt><b>Conf:</b> <code>hashtags_style</code></dt>
<dd>
Configures the text style for displaying hashtags in the task list. Hashtags are identified by their `#HashtagName` syntax.
</dd>
<br>

<dt><b>Flag:</b> <code>--category-style</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_CATEGORY_STYLE</code></dt>
<dt><b>Conf:</b> <code>category_style</code></dt>
<dd>
Defines the default text style for displaying projects, contexts, and hashtags. This serves as the base style and is overridden by specific configurations for individual categories.
</dd>
<br>

<dt><b>Flag:</b> <code>--category-select-style</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_CATEGORY_SELECT_STYLE</code></dt>
<dt><b>Conf:</b> <code>category_select_style</code></dt>
<dd>
Specifies the text style applied to categories when they are selected for filtering.

**default:**
```toml
fg = "Green"
```
</dd>
<br>

<dt><b>Flag:</b> <code>--category-remove-style</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_CATEGORY_REMOVE_STYLE</code></dt>
<dt><b>Conf:</b> <code>category_remove_style</code></dt>
<dd>
Specifies the text style applied to categories that are filtered out from the view.

**default:**
```toml
fg = "Red"
```
</dd>
<br>

<dt><b>Flag:</b> <code>--custom-category-style</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_CUSTOM_CATEGORY_STYLE</code></dt>
<dt><b>Conf:</b> <code>custom_category_style</code></dt>
<dd>
Allows custom text styles to be applied to specific categories by name. Note: Custom styles defined here will override all other category-specific styles, including `category_style`, `category_select_style`, and `category_remove_style`

**default:**
```toml
"+todo-tui".fg = "LightBlue"
```
</dd>
<br>

<dt><b>Flag:</b> <code>--highlight</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_HIGHLIGHT</code></dt>
<dt><b>Conf:</b> <code>highlight</code></dt>
<dd>
Specifies the text style for highlighting elements in the task list that match a search query.

**default:**
```toml
bg = "Yellow"
```
</dd>
<br>
