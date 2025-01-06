# File worker

<dt><b>Flag:</b> <code>--todo-path</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_TODO_PATH</code></dt>
<dt><b>Conf:</b> <code>todo_path</code></dt>
<dd>
Specifies the path to the `todo.txt` file that serves as the main task database. This file contains all your active tasks and is essential for managing your to-do list within the application.

**default:** `$HOME/todo.txt`
</dd>
<br>

<dt><b>Flag:</b> <code>--archive-path</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_ARCHIVE_PATH</code></dt>
<dt><b>Conf:</b> <code>archive_path</code></dt>
<dd>
Specifies the path to the `archive.txt` file, where completed tasks are stored. If not provided, completed tasks are moved to the `todo.txt` file instead of being saved in a separate archive file.
</dd>
<br>

<dt><b>Flag:</b> <code>-d</code>, <code>--autosave-duration</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_AUTOSAVE_DURATION</code></dt>
<dt><b>Conf:</b> <code>autosave_duration</code></dt>
<dd>
Defines the interval, in seconds, for automatic saving of changes to the `todo.txt` file. Is used only if `save_policy` is set to `AutoSave`.

**default:** `900`
</dd>
<br>

<dt><b>Flag:</b> <code>-f</code>, <code>--file-watcher</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_INIT_WIDGET</code></dt>
<dt><b>Conf:</b> <code>file_watcher</code></dt>
<dd>
Enables or disables the file-watcher functionality. When enabled, the application monitors the `todo.txt` (_and optionally `archive.txt`_) file for external changes and automatically reloads.

**default:** `true`
</dd>
<br>

<dt><b>Flag:</b> <code>-i</code>, <code>--save-policy</code></dt>
<dt><b>Env:</b> <code>$TODOTXT_TUI_SAVE_POLICY</code></dt>
<dt><b>Conf:</b> <code>save_policy</code></dt>
<dd>
Determines the policy for saving changes to the todo.txt file and, optionally, the archive.txt file.

- `ManualOnly`: Requires explicit user action to save changes.
- `AutoSave`: Automatically saves changes at intervals defined by `autosave_duration.
- `OnChange`: Saves changes immediately whenever modifications occur.

**default:** `OnChange`

</dd>
<br>
