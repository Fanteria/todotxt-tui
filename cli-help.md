Todo.txt TUI is a highly customizable terminal-based application for managing your todo tasks. It follows
the todo.txt format and offers a wide range of configuration options to suit your needs.

Usage: todotxt-tui [OPTIONS]

Options:
  -c, --config-path <PATH>
          Path to configuration file

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Ui:
  -i, --init-widget <WIDGET_TYPE>
          The widget that will be active when the application starts
          
          [env: TODOTXT_TUI_INIT_WIDGET=]
          [possible values: list, done, project, context, hashtag, preview]

  -t, --window-title <STRING>
          The title of the window when `todotxt-tui` is opened
          
          [env: TODOTXT_TUI_WINDOW_TITLE=]

  -W, --window-keybinds <KEYBINDS>
          Keybindings configured for interacting with the application window
          
          [env: TODOTXT_TUI_WINDOW_KEYBINDS=]

  -R, --list-refresh-rate <DURATION>
          The refresh rate for the list display, in seconds
          
          [env: TODOTXT_TUI_LIST_REFRESH_RATE=]

  -S, --save-state-path <PATH>
          Path to save the application's state (currently unused)
          
          [env: TODOTXT_TUI_SAVE_STATE_PATH=]

  -l, --layout <STRING>
          The layout setting allows you to define a custom layout for the application using blocks `[]`.
          You can specify the orientation of the blocks as either `Direction: Vertical` or `Direction:
          Horizontal`, along with the size of each block as a percentage or value. Within these blocks,
          you can include various widgets, such as:
          
          - `List`: The main list of tasks.
          - `Preview`: The task preview section.
          - `Done`: The list of completed tasks.
          - `Projects`: The list of projects.
          - `Contexts`: The list of contexts.
          - `Hashtags`: The list of hashtags.
          
          [env: TODOTXT_TUI_LAYOUT=]

      --paste-behavior <PASTE_BEHAVIOR>
          Determines how pasted content is processed.
          
          Option as-keys simulates typing the pasted content as if entered via the keyboard. Option
          insert directly inserts the pasted content at the cursor position. Option none disables pasting
          altogether.
          
          [env: TODOTXT_TUI_PASTE_BEHAVIOR=]
          [possible values: as-keys, insert, none]

      --enable-mouse <BOOL>
          Enables or disables mouse interaction support
          
          [env: TODOTXT_TUI_ENABLE_MOUSE=]
          [possible values: true, false]

ToDo:
      --use-done <BOOL>
          Determines whether projects, contexts, and tags from completed tasks should be included in the
          lists of available projects, contexts, and tags
          
          [env: TODOTXT_TUI_USE_DONE=]
          [possible values: true, false]

      --pending-sort <TASK_SORT>
          Sorting option to apply to pending tasks
          
          [env: TODOTXT_TUI_PENDING_SORT=]
          [possible values: none, reverse, priority, alphanumeric, alphanumeric-reverse]

      --done-sort <TASK_SORT>
          Sorting option to apply to completed tasks
          
          [env: TODOTXT_TUI_DONE_SORT=]
          [possible values: none, reverse, priority, alphanumeric, alphanumeric-reverse]

      --delete-final-date <BOOL>
          Specifies whether to delete the final date (if it exists) when a task is moved from completed
          back to pending
          
          [env: TODOTXT_TUI_DELETE_FINAL_DATE=]
          [possible values: true, false]

      --set-final-date <SET_FINAL_DATE>
          Configures how the final date is handled when a task is marked as completed. Options include
          overriding the date, only adding it if missing, or never setting it
          
          [env: TODOTXT_TUI_SET_FINAL_DATE=]
          [possible values: override, only-missing, never]

FileWorker:
      --todo-path <PATH>
          The path to your `todo.txt` file, which stores your task list
          
          [env: TODOTXT_TUI_TODO_PATH=]

      --archive-path <PATH>
          The path to your `archive.txt` file, where completed tasks are stored. If not provided,
          completed tasks will be archived within your `todo.txt` file
          
          [env: TODOTXT_TUI_ARCHIVE_PATH=]

  -d, --autosave-duration <DURATION>
          The duration (in seconds) between automatic saves of the `todo.txt` file
          
          [env: TODOTXT_TUI_AUTOSAVE_DURATION=]

  -f, --file-watcher <BOOL>
          Enable or disable the file watcher, which automatically reloads the `todo.txt` file when
          changes are detected
          
          [env: TODOTXT_TUI_FILE_WATCHER=]
          [possible values: true, false]

      --save-policy <SAVE_POLICY>
          The save policy for how and when the `todo.txt` and optionally `archive.txt` files should be
          saved
          
          [env: TODOTXT_TUI_SAVE_POLICY=]
          [possible values: manual-only, auto-save, on-change]

WidgetBase:
  -T, --tasks-keybind <KEYBINDS>
          Keybindings configured for interacting with tasks
          
          [env: TODOTXT_TUI_TASKS_KEYBIND=]

  -C, --category-keybind <KEYBINDS>
          Keybindings configured for interacting with categories
          
          [env: TODOTXT_TUI_CATEGORY_KEYBIND=]

      --border-type <BORDER_TYPE>
          The type of border style to use for the UI widgets
          
          [env: TODOTXT_TUI_BORDER_TYPE=]
          [possible values: plain, rounded, double, thick]

List:
  -s, --list-shift <+NUM>
          The number of lines displayed above and below the currently active item in a list when the list
          is moving
          
          [env: TODOTXT_TUI_LIST_SHIFT=]

  -L, --list-keybind <KEYBINDS>
          Keybindings configured for interacting with lists
          
          [env: TODOTXT_TUI_LIST_KEYBIND=]

      --pending-format <STRING>
          The format string used to render pending tasks in the list
          
          [env: TODOTXT_TUI_PENDING_FORMAT=]

      --done-format <STRING>
          The format string used to render completed tasks in the list
          
          [env: TODOTXT_TUI_DONE_FORMAT=]

Preview:
  -p, --preview-format <STRING>
          The format string used to generate the preview, supporting placeholders for dynamic content
          
          [env: TODOTXT_TUI_PREVIEW_FORMAT=]

  -w, --wrap-preview <BOOL>
          Determines whether long lines in the preview should be wrapped to fit within the available
          width
          
          [env: TODOTXT_TUI_WRAP_PREVIEW=]
          [possible values: true, false]

ActiveColor:
  -A, --list-active-color <TEXT_STYLE>
          The text style used to highlight the active item in a list
          
          [env: TODOTXT_TUI_LIST_ACTIVE_COLOR=]

  -P, --pending-active-color <TEXT_STYLE>
          The text style used to highlight an active task that is in the pending list. This option
          overrides the `list_active_color`
          
          [env: TODOTXT_TUI_PENDING_ACTIVE_COLOR=]

  -D, --done-active-color <TEXT_STYLE>
          The text style used to highlight an active task that is in the completed list. This option
          overrides the `list_active_color`
          
          [env: TODOTXT_TUI_DONE_ACTIVE_COLOR=]

      --category-active-color <TEXT_STYLE>
          The text style used to highlight an active category. This option overrides the
          `list_active_color`
          
          [env: TODOTXT_TUI_CATEGORY_ACTIVE_COLOR=]

      --projects-active-color <TEXT_STYLE>
          The text style used to highlight an active project. This option overrides the
          `category_active_color`
          
          [env: TODOTXT_TUI_PROJECTS_ACTIVE_COLOR=]

      --contexts-active-color <TEXT_STYLE>
          The text style used to highlight an active context. This option overrides the
          `category_active_color`
          
          [env: TODOTXT_TUI_CONTEXTS_ACTIVE_COLOR=]

      --tags-active-color <TEXT_STYLE>
          The text style used to highlight an active tag. This option overrides the
          `category_active_color`
          
          [env: TODOTXT_TUI_TAGS_ACTIVE_COLOR=]

Styles:
      --active-color <COLOR>
          Defines the color used to highlight the active window
          
          [env: TODOTXT_TUI_ACTIVE_COLOR=]

      --priority-style <TEXT_STYLE_LIST>
          A list of text styles applied to tasks based on their priority levels
          
          [env: TODOTXT_TUI_PRIORITY_STYLE=]

      --projects-style <TEXT_STYLE>
          Specifies the text style used for displaying projects within task lists
          
          [env: TODOTXT_TUI_PROJECTS_STYLE=]

      --contexts-style <TEXT_STYLE>
          Specifies the text style used for displaying contexts (e.g., @home, @work) within task lists
          
          [env: TODOTXT_TUI_CONTEXTS_STYLE=]

      --hashtags-style <TEXT_STYLE>
          Specifies the text style used for displaying hashtags within task lists. Note: This style is
          overridden by custom styles defined for specific categories
          
          [env: TODOTXT_TUI_HASHTAGS_STYLE=]

      --category-style <TEXT_STYLE>
          Defines the default text style for displaying projects, contexts, and hashtags within task
          lists. Note: This style is overridden by specific styles for individual categories
          
          [env: TODOTXT_TUI_CATEGORY_STYLE=]

      --category-select-style <TEXT_STYLE>
          Specifies the text style applied to categories when they are selected for filtering
          
          [env: TODOTXT_TUI_CATEGORY_SELECT_STYLE=]

      --category-remove-style <TEXT_STYLE>
          Specifies the text style applied to categories that are filtered out from the view
          
          [env: TODOTXT_TUI_CATEGORY_REMOVE_STYLE=]

      --custom-category-style <CUSTOM_CATEGORY_STYLE>
          Allows custom text styles to be applied to specific categories by name. Note: Custom styles
          defined here will override all other category-specific styles, including `category_style`,
          `category_select_style`, and `category_remove_style`
          
          [env: TODOTXT_TUI_CUSTOM_CATEGORY_STYLE=]

      --highlight <TEXT_STYLE>
          Specifies the text style used to highlight elements that match a search within lists
          
          [env: TODOTXT_TUI_HIGHLIGHT=]

HookPaths:
      --pre-new-task <PATH>
          Path to the script executed before creating a new task. If none, no action is taken before a
          new task is created
          
          [env: TODOTXT_TUI_PRE_NEW_TASK=]

      --post-new-task <PATH>
          Path to the script executed after creating a new task. If none, no action is taken after a new
          task is created
          
          [env: TODOTXT_TUI_POST_NEW_TASK=]

      --pre-remove-task <PATH>
          Path to the script executed before removing a task. If none, no action is taken before a task
          is removed
          
          [env: TODOTXT_TUI_PRE_REMOVE_TASK=]

      --post-remove-task <PATH>
          Path to the script executed after removing a task. If none, no action is taken after a task is
          removed
          
          [env: TODOTXT_TUI_POST_REMOVE_TASK=]

      --pre-move-task <PATH>
          Path to the script executed before moving a task. If none, no action is taken before a task is
          moved
          
          [env: TODOTXT_TUI_PRE_MOVE_TASK=]

      --post-move-task <PATH>
          Path to the script executed after moving a task. If none, no action is taken after a task is
          moved
          
          [env: TODOTXT_TUI_POST_MOVE_TASK=]

      --pre-update-task <PATH>
          Path to the script executed before updating a task. If none, no action is taken before a task
          is updated
          
          [env: TODOTXT_TUI_PRE_UPDATE_TASK=]

      --post-update-task <PATH>
          Path to the script executed after updating a task. If none, no action is taken after a task is
          updated
          
          [env: TODOTXT_TUI_POST_UPDATE_TASK=]

Export:
      --export-autocomplete [<PATH>]
          Generate autocomplete script to given file path. If path is not set, standard output will be
          used

      --export-config [<PATH>]
          Generate full configuration file for actual session so present configuration file and command
          lines options are taken in account. If path is not set, standard output will be used

      --export-default-config [<PATH>]
          Generate configuration file with default values to given file path. If path is not set,
          standard output will be used
