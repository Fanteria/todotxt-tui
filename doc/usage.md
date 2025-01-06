# Usage

Todo.txt TUI is a powerful and flexible terminal-based interface designed to simplify task management. Its customization options allow you to tailor the tool to fit your workflow. Below, you'll find a comprehensive list of keybindings and actions, organized by category, to help you get started. All actions can be customized in the configuration file to suit your preferences.

## Keybindings

**Navigation:**
- `j`: Move down to the next task in the list.
- `k`: Move up to the previous task in the list.
- `g`: Jump directly to the first task in the list.
- `G`: Jump directly to the last task in the list.

**Managing Tasks:**

- `Enter`: Show task in preview.
- `U`: Move the currently selected task up in the list.
- `D`: Move the currently selected task down in the list.
- `x`: Remove the selected task permanently from the list.
- `d`: Toggle a task's status between pending and done.
- `I`: Input a new task into the list.
- `E`: Edit the currently selected task to update its details.

**Widget Navigation:**

- `J`: Jump to the widget located below the current one.
- `K`: Jump to the widget located above the current one.
- `H`: Move to the widget on the left side of the current one.
- `L`: Move to the widget on the right side of the current one.

**Filtering and Searching Tasks:**

- `Backspace`: Apply a filter to display specific categories in the pending or done lists.
- `/`: Search for a keyword or phrase within the current list.
- `n`: Navigate to the next occurrence of the search term.
- `N`: Navigate to the previous occurrence of the search term.
- `h`: Clear the current search term and reset the list view.

**File Operations:**

- `S`: Save all current tasks to the todo.txt file.
- `u`: Reload tasks from the todo.txt file to update the list with external changes.

**Quit:**
- `q`: Quit Todo.txt TUI and save any unsaved changes.

## Task Input Bar

The task input bar allows you to add or edit tasks. Press `Enter` to confirm the input and save the task, or press `Esc` to cancel. Additionally, common Bash/Emacs keybindings are supported:

- `Ctrl+w`: Delete the previous word.
- `Ctrl+e`: Move the cursor to the end of the line.
- `Ctrl+a`: Move the cursor to the beginning of the line.
- `Ctrl+k`: Delete from the cursor to the end of the line.
- `Ctrl+u`: Delete from the cursor to the beginning of the line.

