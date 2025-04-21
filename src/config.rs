mod colors;
mod keycode;
mod options;
mod parsers;
mod styles;
mod text_style;
mod traits;

pub use self::{
    colors::Color,
    keycode::KeyCodeDef,
    options::{
        PasteBehavior, SavePolicy, SetFinalDateType, TaskSort, TextModifier, WidgetBorderType,
    },
    styles::CustomCategoryStyle,
    text_style::{TextStyle, TextStyleList},
    traits::{Conf, ConfMerge, ConfigDefaults},
};

use crate::{
    layout::widget::WidgetType,
    todo::{ToDoCategory, ToDoData},
    ui::{EventHandlerUI, KeyShortcut, UIEvent},
    Result,
};
use clap::{builder::styling::AnsiColor, FromArgMatches};
use crossterm::event::{KeyCode, KeyModifiers};
use std::{env::var, path::PathBuf, time::Duration};
use todotxt_tui_macros::{Conf, ConfMerge};
use tui::style::Color as tuiColor;

#[derive(Conf, Clone, Debug, PartialEq, Eq)]
pub struct FileWorkerConfig {
    /// The path to your `todo.txt` file, which stores your task list.
    pub todo_path: PathBuf,
    /// The path to your `archive.txt` file, where completed tasks are stored.
    /// If not provided, completed tasks will be archived within your `todo.txt` file.
    pub archive_path: Option<PathBuf>,
    /// The duration (in seconds) between automatic saves of the `todo.txt` file.
    #[arg(short = 'd')]
    pub autosave_duration: Duration,
    /// Enable or disable the file watcher, which automatically reloads the `todo.txt` file
    /// when changes are detected.
    #[arg(short = 'f')]
    pub file_watcher: bool,
    /// The save policy for how and when the `todo.txt` and optionally `archive.txt` files
    /// should be saved.
    pub save_policy: SavePolicy,
}

impl Default for FileWorkerConfig {
    fn default() -> Self {
        Self {
            todo_path: PathBuf::from(var("HOME").unwrap_or(String::from("~")) + "/todo.txt"),
            archive_path: None,
            autosave_duration: Duration::from_secs(900),
            file_watcher: true,
            save_policy: SavePolicy::default(),
        }
    }
}

#[derive(Conf, Clone, Debug, PartialEq, Eq)]
pub struct ActiveColorConfig {
    /// The text style used to highlight the active item in a list.
    #[arg(short = 'A')]
    list_active_color: TextStyle,
    /// The text style used to highlight an active task that is in the pending list.
    /// This option overrides the `list_active_color`.
    #[arg(short = 'P')]
    pending_active_color: TextStyle,
    /// The text style used to highlight an active task that is in the completed list.
    /// This option overrides the `list_active_color`.
    #[arg(short = 'D')]
    done_active_color: TextStyle,
    /// The text style used to highlight an active category.
    /// This option overrides the `list_active_color`.
    category_active_color: TextStyle,
    /// The text style used to highlight an active project.
    /// This option overrides the `category_active_color`.
    projects_active_color: TextStyle,
    /// The text style used to highlight an active context.
    /// This option overrides the `category_active_color`.
    contexts_active_color: TextStyle,
    /// The text style used to highlight an active tag.
    /// This option overrides the `category_active_color`.
    tags_active_color: TextStyle,
}

impl ActiveColorConfig {
    /// Retrieves the active style for a given `ToDoData` type, combining it with
    /// the list's active color.
    pub fn get_active_style(&self, data_type: &ToDoData) -> TextStyle {
        self.list_active_color.combine(&match data_type {
            ToDoData::Done => self.done_active_color,
            ToDoData::Pending => self.pending_active_color,
        })
    }

    /// Returns the active configuration style for a given category.
    /// This function combines three color settings based on the specified `ToDoCategory`:
    /// - The list active color.
    /// - The category specific active color (projects, contexts, or hashtags).
    pub fn get_active_config_style(&self, category: &ToDoCategory) -> TextStyle {
        self.list_active_color
            .combine(&self.category_active_color)
            .combine(match category {
                ToDoCategory::Projects => &self.projects_active_color,
                ToDoCategory::Contexts => &self.contexts_active_color,
                ToDoCategory::Hashtags => &self.tags_active_color,
            })
    }
}

impl Default for ActiveColorConfig {
    fn default() -> Self {
        Self {
            list_active_color: TextStyle::default().bg(Color::lightred()),
            pending_active_color: TextStyle::default(),
            done_active_color: TextStyle::default(),
            category_active_color: TextStyle::default(),
            projects_active_color: TextStyle::default(),
            contexts_active_color: TextStyle::default(),
            tags_active_color: TextStyle::default(),
        }
    }
}

#[derive(Conf, Clone, Debug, PartialEq, Eq)]
pub struct ListConfig {
    /// The number of lines displayed above and below the currently active
    /// item in a list when the list is moving.
    #[arg(short = 's')]
    pub list_shift: usize,
    /// Keybindings configured for interacting with lists.
    #[arg(short = 'L')]
    pub list_keybind: EventHandlerUI,
    /// The format string used to render pending tasks in the list.
    pub pending_format: String,
    /// The format string used to render completed tasks in the list.
    pub done_format: String,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            list_shift: 4,
            list_keybind: EventHandlerUI::from([
                (KeyShortcut::from(KeyCode::Char('j')), UIEvent::ListDown),
                (KeyShortcut::from(KeyCode::Char('k')), UIEvent::ListUp),
                (KeyShortcut::from(KeyCode::Char('g')), UIEvent::ListFirst),
                (
                    KeyShortcut::new(KeyCode::Char('g'), KeyModifiers::SHIFT),
                    UIEvent::ListLast,
                ),
                (KeyShortcut::from(KeyCode::Char('h')), UIEvent::CleanSearch),
            ]),
            pending_format: String::from("[$subject](! priority)"),
            done_format: String::from("[$subject](! priority)"),
        }
    }
}

#[derive(Conf, Clone, Debug, PartialEq, Eq)]
pub struct PreviewConfig {
    /// The format string used to generate the preview, supporting placeholders
    /// for dynamic content.
    #[arg(short = 'p')]
    pub preview_format: String,
    /// Determines whether long lines in the preview should be wrapped to fit
    /// within the available width.
    #[arg(short = 'w')]
    pub wrap_preview: bool,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            preview_format: String::from(
                "Pending: $pending Done: $done
Subject: $subject
Priority: $priority
Create date: $create_date
Link: $link",
            ),
            wrap_preview: true,
        }
    }
}

#[derive(Conf, Clone, Debug, PartialEq, Eq)]
pub struct ToDoConfig {
    /// Determines whether projects, contexts, and tags from completed tasks
    /// should be included in the lists of available projects, contexts, and tags.
    pub use_done: bool,
    /// Sorting option to apply to pending tasks.
    pub pending_sort: TaskSort,
    /// Sorting option to apply to completed tasks.
    pub done_sort: TaskSort,
    /// Specifies whether to delete the final date (if it exists) when a task is moved from completed back to pending.
    pub delete_final_date: bool,
    /// Configures how the final date is handled when a task is marked as completed.
    /// Options include overriding the date, only adding it if missing, or never setting it.
    pub set_final_date: SetFinalDateType,
}

impl Default for ToDoConfig {
    fn default() -> Self {
        Self {
            use_done: false,
            pending_sort: TaskSort::default(),
            done_sort: TaskSort::default(),
            delete_final_date: true,
            set_final_date: SetFinalDateType::default(),
        }
    }
}

#[derive(Conf, Clone, Debug, PartialEq, Eq)]
pub struct UiConfig {
    /// The widget that will be active when the application starts.
    #[arg(short = 'i')]
    pub init_widget: WidgetType,
    /// The title of the window when `todotxt-tui` is opened.
    #[arg(short = 't')]
    pub window_title: String,
    /// Keybindings configured for interacting with the application window.
    #[arg(short = 'W')]
    pub window_keybinds: EventHandlerUI,
    /// The refresh rate for the list display, in seconds.
    #[arg(short = 'R')]
    pub list_refresh_rate: Duration,
    /// Path to save the application's state (currently unused).
    #[arg(short = 'S')]
    pub save_state_path: Option<PathBuf>,
    /// The layout setting allows you to define a custom layout for the application using blocks `[]`. You can specify the orientation of the blocks as either `Direction: Vertical` or `Direction: Horizontal`, along with the size of each block as a percentage or value. Within these blocks, you can include various widgets, such as:
    ///
    /// - `List`: The main list of tasks.
    /// - `Preview`: The task preview section.
    /// - `Done`: The list of completed tasks.
    /// - `Projects`: The list of projects.
    /// - `Contexts`: The list of contexts.
    /// - `Hashtags`: The list of hashtags.
    #[arg(short = 'l', verbatim_doc_comment)]
    pub layout: String,
    /// Determines how pasted content is processed.
    ///
    /// Option as-keys simulates typing the pasted content as if entered via the keyboard.
    /// Option insert directly inserts the pasted content at the cursor position.
    /// Option none disables pasting altogether.
    pub paste_behavior: PasteBehavior,
    /// Enables or disables mouse interaction support.
    pub enable_mouse: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            init_widget: WidgetType::List,
            window_title: String::from("ToDo tui"),
            window_keybinds: EventHandlerUI::from([
                (KeyShortcut::from(KeyCode::Char('q')), UIEvent::Quit),
                (
                    KeyShortcut::new(KeyCode::Char('s'), KeyModifiers::SHIFT),
                    UIEvent::Save,
                ),
                (KeyShortcut::from(KeyCode::Char('u')), UIEvent::Load),
                (
                    KeyShortcut::new(KeyCode::Char('h'), KeyModifiers::SHIFT),
                    UIEvent::MoveLeft,
                ),
                (
                    KeyShortcut::new(KeyCode::Char('l'), KeyModifiers::SHIFT),
                    UIEvent::MoveRight,
                ),
                (
                    KeyShortcut::new(KeyCode::Char('k'), KeyModifiers::SHIFT),
                    UIEvent::MoveUp,
                ),
                (
                    KeyShortcut::new(KeyCode::Char('j'), KeyModifiers::SHIFT),
                    UIEvent::MoveDown,
                ),
                (
                    KeyShortcut::new(KeyCode::Char('i'), KeyModifiers::SHIFT),
                    UIEvent::InsertMode,
                ),
                (
                    KeyShortcut::new(KeyCode::Char('e'), KeyModifiers::SHIFT),
                    UIEvent::EditMode,
                ),
                (KeyShortcut::from(KeyCode::Char('/')), UIEvent::SearchMode),
            ]),
            list_refresh_rate: Duration::from_secs(5),
            save_state_path: None,
            layout: String::from(concat!(
                "[",
                "  Direction: Horizontal,",
                "  Size: 50%,",
                "  [",
                "    List: 80%, Preview: 20%,",
                "  ],",
                "  [",
                "    Direction: Vertical,",
                "    Done: 60%,",
                "    [",
                "      Contexts: 50%,",
                "      Projects: 50%,",
                "    ],",
                "  ],",
                "]",
            )),
            paste_behavior: Default::default(),
            enable_mouse: true,
        }
    }
}

#[derive(Conf, Clone, Debug, PartialEq, Eq)]
pub struct WidgetBaseConfig {
    /// Keybindings configured for interacting with tasks.
    #[arg(short = 'T')]
    pub tasks_keybind: EventHandlerUI,
    /// Keybindings configured for interacting with categories.
    #[arg(short = 'C')]
    pub category_keybind: EventHandlerUI,
    /// The type of border style to use for the UI widgets.
    pub border_type: WidgetBorderType,
}

impl Default for WidgetBaseConfig {
    fn default() -> Self {
        Self {
            tasks_keybind: EventHandlerUI::from([
                (
                    KeyShortcut::new(KeyCode::Char('u'), KeyModifiers::SHIFT),
                    UIEvent::SwapUpItem,
                ),
                (
                    KeyShortcut::new(KeyCode::Char('d'), KeyModifiers::SHIFT),
                    UIEvent::SwapDownItem,
                ),
                (KeyShortcut::from(KeyCode::Char('x')), UIEvent::RemoveItem),
                (KeyShortcut::from(KeyCode::Char('d')), UIEvent::MoveItem),
                (KeyShortcut::from(KeyCode::Enter), UIEvent::Select),
                (KeyShortcut::from(KeyCode::Char('n')), UIEvent::NextSearch),
                (
                    KeyShortcut::new(KeyCode::Char('n'), KeyModifiers::SHIFT),
                    UIEvent::PrevSearch,
                ),
            ]),
            category_keybind: EventHandlerUI::from([
                (KeyShortcut::from(KeyCode::Enter), UIEvent::Select),
                (KeyShortcut::from(KeyCode::Backspace), UIEvent::Remove),
                (KeyShortcut::from(KeyCode::Char('n')), UIEvent::NextSearch),
                (
                    KeyShortcut::new(KeyCode::Char('n'), KeyModifiers::SHIFT),
                    UIEvent::PrevSearch,
                ),
            ]),
            border_type: WidgetBorderType::default(),
        }
    }
}

#[derive(Conf, Clone, Debug, PartialEq, Eq)]
pub struct Styles {
    /// Defines the color used to highlight the active window.
    pub active_color: Color,
    /// A list of text styles applied to tasks based on their priority levels.
    pub priority_style: TextStyleList,
    /// Specifies the text style used for displaying projects within task lists.
    pub projects_style: TextStyle,
    /// Specifies the text style used for displaying contexts (e.g., @home, @work)
    /// within task lists.
    pub contexts_style: TextStyle,
    /// Specifies the text style used for displaying hashtags within task lists.
    /// Note: This style is overridden by custom styles defined for specific categories.
    pub hashtags_style: TextStyle,
    /// Defines the default text style for displaying projects, contexts,
    /// and hashtags within task lists.
    /// Note: This style is overridden by specific styles for individual categories.
    pub category_style: TextStyle,
    /// Specifies the text style applied to categories when they are selected for filtering.
    pub category_select_style: TextStyle,
    /// Specifies the text style applied to categories that are filtered out from the view.
    pub category_remove_style: TextStyle,
    /// Allows custom text styles to be applied to specific categories by name.
    /// Note: Custom styles defined here will override all other category-specific styles,
    /// including `category_style`, `category_select_style`, and `category_remove_style`.
    pub custom_category_style: CustomCategoryStyle,
    /// Specifies the text style used to highlight elements that match a search
    /// within lists.
    pub highlight: TextStyle,
}

impl Styles {
    /// Retrieves the text style for a specified category. If a custom style
    /// has been defined for the category, it will be used; otherwise,
    /// the base style for that category is employed.
    pub fn get_category_style(&self, category: &str) -> TextStyle {
        match self.custom_category_style.get(category) {
            Some(style) => *style,
            None => self.get_category_base_style(category),
        }
    }

    /// Retrieves the base style for a specified category based on its initial
    /// character: '+' for projects, '@' for contexts, and '#' for hashtags.
    /// If the category does not match any of these prefixes, it defaults
    /// to the general `category_style`.
    fn get_category_base_style(&self, category: &str) -> TextStyle {
        match category.chars().next().unwrap() {
            '+' => self.category_style.combine(&self.projects_style),
            '@' => self.category_style.combine(&self.contexts_style),
            '#' => self.category_style.combine(&self.hashtags_style),
            _ => self.category_style,
        }
    }
}

impl Default for Styles {
    fn default() -> Self {
        let mut custom_category_style = CustomCategoryStyle::default();
        custom_category_style.insert(
            String::from("+todo-tui"),
            TextStyle::default().fg(Color::lightblue()),
        );
        Self {
            active_color: Color(tuiColor::Red),
            priority_style: TextStyleList::default(),
            projects_style: TextStyle::default(),
            contexts_style: TextStyle::default(),
            hashtags_style: TextStyle::default(),
            category_style: TextStyle::default(),
            category_select_style: TextStyle::default().fg(Color::green()),
            category_remove_style: TextStyle::default().fg(Color::red()),
            custom_category_style,
            highlight: TextStyle::default().bg(Color::yellow()),
        }
    }
}

#[derive(Conf, Clone, Debug, PartialEq, Eq, Default)]
pub struct HookPaths {
    /// Path to the script executed before creating a new task.
    /// If none, no action is taken before a new task is created.
    pub pre_new_task: Option<PathBuf>,
    /// Path to the script executed after creating a new task.
    /// If none, no action is taken after a new task is created.
    pub post_new_task: Option<PathBuf>,
    /// Path to the script executed before removing a task.
    /// If none, no action is taken before a task is removed.
    pub pre_remove_task: Option<PathBuf>,
    /// Path to the script executed after removing a task.
    /// If none, no action is taken after a task is removed.
    pub post_remove_task: Option<PathBuf>,
    /// Path to the script executed before moving a task.
    /// If none, no action is taken before a task is moved.
    pub pre_move_task: Option<PathBuf>,
    /// Path to the script executed after moving a task.
    /// If none, no action is taken after a task is moved.
    pub post_move_task: Option<PathBuf>,
    /// Path to the script executed before updating a task.
    /// If none, no action is taken before a task is updated.
    pub pre_update_task: Option<PathBuf>,
    /// Path to the script executed after updating a task.
    /// If none, no action is taken after a task is updated.
    pub post_update_task: Option<PathBuf>,
}

#[derive(ConfMerge, Default, Debug, PartialEq, Eq)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    pub ui_config: UiConfig,
    pub todo_config: ToDoConfig,
    pub file_worker_config: FileWorkerConfig,
    pub widget_base_config: WidgetBaseConfig,
    pub list_config: ListConfig,
    pub preview_config: PreviewConfig,
    pub active_color_config: ActiveColorConfig,
    pub styles: Styles,
    pub hook_paths: HookPaths,
}

impl Config {
    pub fn config_folder() -> PathBuf {
        match var("XDG_CONFIG_HOME") {
            Ok(config_path) => PathBuf::from(config_path),
            Err(_) => PathBuf::from(var("HOME").unwrap_or(String::from("~"))).join(".config"),
        }
        .join(env!("CARGO_PKG_NAME"))
    }
}

impl ConfigDefaults for Config {
    fn config_path() -> PathBuf {
        Self::config_folder().join("todotxt-tui.toml")
    }

    fn help_colors() -> clap::builder::Styles {
        clap::builder::Styles::styled()
            .usage(AnsiColor::Green.on_default().bold())
            .literal(AnsiColor::Cyan.on_default().bold())
            .header(AnsiColor::Green.on_default().bold())
            .invalid(AnsiColor::Yellow.on_default())
            .error(AnsiColor::Red.on_default().bold())
            .valid(AnsiColor::Green.on_default())
            .placeholder(AnsiColor::Cyan.on_default())
    }
}

#[cfg(test)]
mod tests {
    use self::parsers::*;
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{path::PathBuf, time::Duration};
    use test_log::test;

    pub fn get_test_dir() -> String {
        var("TODO_TUI_TEST_DIR").unwrap()
    }

    pub fn get_test_file(name: &str) -> PathBuf {
        let path = PathBuf::from(get_test_dir()).join(name);
        log::trace!("Get test file {path:?}");
        path
    }

    #[test]
    fn test_deserialization() {
        let deserialized = Config::from_reader(
            r#"
            active_color = "Green"
            init_widget = "Done"
        "#
            .as_bytes(),
        )
        .unwrap();

        assert_eq!(*deserialized.styles.active_color, tuiColor::Green);
        assert_eq!(deserialized.ui_config.init_widget, WidgetType::Done);
        assert_eq!(
            deserialized.ui_config.window_title,
            UiConfig::default().window_title
        );
    }

    #[test]
    fn get_active_style() {
        {
            let color = ActiveColorConfig {
                list_active_color: TextStyle::default().bg(Color::red()),
                pending_active_color: TextStyle::default().bg(Color::yellow()),
                ..Default::default()
            };
            assert_eq!(
                color.get_active_style(&ToDoData::Pending),
                TextStyle::default().bg(Color::yellow())
            );
        }

        {
            let color = ActiveColorConfig {
                list_active_color: TextStyle::default().bg(Color::red()),
                ..Default::default()
            };
            assert_eq!(
                color.get_active_style(&ToDoData::Pending),
                TextStyle::default().bg(Color::red())
            );
        }

        {
            let color = ActiveColorConfig {
                list_active_color: TextStyle::default().bg(Color::green()).fg(Color::blue()),
                done_active_color: TextStyle::default()
                    .fg(Color::black())
                    .modifier(TextModifier::Bold),
                ..Default::default()
            };
            assert_eq!(
                color.get_active_style(&ToDoData::Done),
                TextStyle::default()
                    .bg(Color::green())
                    .fg(Color::black())
                    .modifier(TextModifier::Bold)
            );
        }
    }

    #[test]
    fn get_active_config_style() {
        let color = ActiveColorConfig {
            list_active_color: TextStyle::default().bg(Color::red()),
            category_active_color: TextStyle::default().fg(Color::white()),
            ..Default::default()
        };
        assert_eq!(
            color.get_active_config_style(&ToDoCategory::Projects),
            TextStyle::default().bg(Color::red()).fg(Color::white())
        );
    }

    #[test]
    fn test_load() -> Result<()> {
        let s = r#"
        active_color = "Blue"
        window_title = "Title"
        todo_path = "path to todo file"
        "#;

        let default = Config::default();
        let c = Config::from_reader(s.as_bytes())?;
        assert_eq!(*c.styles.active_color, tuiColor::Blue);
        assert_eq!(c.ui_config.init_widget, default.ui_config.init_widget);
        assert_eq!(c.ui_config.window_title, String::from("Title"));
        assert_eq!(
            c.file_worker_config.todo_path,
            PathBuf::from("path to todo file")
        );
        assert_eq!(c.file_worker_config.archive_path, None);

        Ok(())
    }

    #[test]
    fn help_can_be_generated() -> Result<()> {
        Config::from_args(Vec::<&str>::new())?;
        Ok(())
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("1000"), Ok(Duration::from_secs(1000)));
        assert!(parse_duration("-1000").is_err());
    }

    #[test]
    fn empty_config() -> Result<()> {
        let empty_config = get_test_file("empty_config.toml");
        let default = Config::from_file(empty_config)?;
        assert_eq!(default, Config::default());

        Ok(())
    }

    #[test]
    fn changed_config() -> Result<()> {
        let testing_config = get_test_file("testing_config.toml");
        let config = Config::from_file(testing_config)?;
        let mut expected = Config::default();
        expected.styles.active_color = Color::blue();
        expected.ui_config.init_widget = WidgetType::Project;
        expected.ui_config.window_title = String::from("Window title");
        expected.ui_config.layout = String::from("Short invalid layout");
        expected.file_worker_config.todo_path = PathBuf::from("invalid/path/to/todo.txt");
        expected.file_worker_config.archive_path =
            Some(PathBuf::from("invalid/path/to/archive.txt"));
        expected.file_worker_config.file_watcher = false;
        expected.list_config.list_shift = 0;
        expected.todo_config.use_done = true;
        expected.todo_config.pending_sort = TaskSort::Priority;
        expected.todo_config.done_sort = TaskSort::Reverse;
        expected.todo_config.delete_final_date = false;
        expected.todo_config.set_final_date = SetFinalDateType::Never;
        expected.preview_config.preview_format = String::from("unimportant preview");
        expected.preview_config.wrap_preview = false;
        expected.ui_config.window_keybinds = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('e')), UIEvent::EditMode),
            (KeyShortcut::from(KeyCode::Char('q')), UIEvent::Quit),
            (
                KeyShortcut::new(KeyCode::Char('s'), KeyModifiers::SHIFT),
                UIEvent::Save,
            ),
            (KeyShortcut::from(KeyCode::Char('u')), UIEvent::Load),
            (
                KeyShortcut::new(KeyCode::Char('h'), KeyModifiers::SHIFT),
                UIEvent::MoveLeft,
            ),
            (
                KeyShortcut::new(KeyCode::Char('l'), KeyModifiers::SHIFT),
                UIEvent::MoveRight,
            ),
            (
                KeyShortcut::new(KeyCode::Char('k'), KeyModifiers::SHIFT),
                UIEvent::MoveUp,
            ),
            (
                KeyShortcut::new(KeyCode::Char('j'), KeyModifiers::SHIFT),
                UIEvent::MoveDown,
            ),
            (
                KeyShortcut::new(KeyCode::Char('i'), KeyModifiers::SHIFT),
                UIEvent::InsertMode,
            ),
            (
                KeyShortcut::new(KeyCode::Char('e'), KeyModifiers::SHIFT),
                UIEvent::EditMode,
            ),
            (KeyShortcut::from(KeyCode::Char('/')), UIEvent::SearchMode),
        ]);
        expected.ui_config.list_refresh_rate = Duration::from_secs(10);
        expected.active_color_config.list_active_color = TextStyle::default().bg(Color::green());
        expected.file_worker_config.autosave_duration = Duration::from_secs(100);
        expected.list_config.list_keybind = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('g')), UIEvent::ListLast),
            (KeyShortcut::from(KeyCode::Char('j')), UIEvent::ListDown),
            (KeyShortcut::from(KeyCode::Char('k')), UIEvent::ListUp),
            (
                KeyShortcut::new(KeyCode::Char('g'), KeyModifiers::SHIFT),
                UIEvent::ListLast,
            ),
            (KeyShortcut::from(KeyCode::Char('h')), UIEvent::CleanSearch),
        ]);
        expected.widget_base_config.tasks_keybind = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('s')), UIEvent::Select),
            (
                KeyShortcut::new(KeyCode::Char('u'), KeyModifiers::SHIFT),
                UIEvent::SwapUpItem,
            ),
            (
                KeyShortcut::new(KeyCode::Char('d'), KeyModifiers::SHIFT),
                UIEvent::SwapDownItem,
            ),
            (KeyShortcut::from(KeyCode::Char('x')), UIEvent::RemoveItem),
            (KeyShortcut::from(KeyCode::Char('d')), UIEvent::MoveItem),
            (KeyShortcut::from(KeyCode::Enter), UIEvent::Select),
            (KeyShortcut::from(KeyCode::Char('n')), UIEvent::NextSearch),
            (
                KeyShortcut::new(KeyCode::Char('n'), KeyModifiers::SHIFT),
                UIEvent::PrevSearch,
            ),
        ]);
        expected.widget_base_config.category_keybind = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('r')), UIEvent::Remove),
            (KeyShortcut::from(KeyCode::Enter), UIEvent::Select),
            (KeyShortcut::from(KeyCode::Backspace), UIEvent::Remove),
            (KeyShortcut::from(KeyCode::Char('n')), UIEvent::NextSearch),
            (
                KeyShortcut::new(KeyCode::Char('n'), KeyModifiers::SHIFT),
                UIEvent::PrevSearch,
            ),
        ]);
        expected.styles.category_select_style = TextStyle::default().fg(Color::red());
        expected.styles.category_remove_style = TextStyle::default().fg(Color::green());
        expected.styles.custom_category_style = CustomCategoryStyle::default();
        expected.styles.custom_category_style.insert(
            String::from("+project"),
            TextStyle::default().fg(Color::green()),
        );

        assert_eq!(config.ui_config, expected.ui_config);
        assert_eq!(config.todo_config, expected.todo_config);
        assert_eq!(config.file_worker_config, expected.file_worker_config);
        assert_eq!(config.widget_base_config, expected.widget_base_config);
        assert_eq!(config.list_config, expected.list_config);
        assert_eq!(config.preview_config, expected.preview_config);
        assert_eq!(config.active_color_config, expected.active_color_config);
        assert_eq!(config.styles, expected.styles);

        Ok(())
    }

    #[test]
    fn default_values_clap() -> Result<()> {
        let empty_config = get_test_file("empty_config.toml");
        let default = Config::from_args(vec![
            "NAME",
            "--config-path",
            empty_config.to_str().unwrap(),
        ])?;
        assert_eq!(default, Config::default());
        Ok(())
    }

    #[test]
    fn custom_clap_arguments() -> Result<()> {
        let testing_config = get_test_file("testing_config.toml");
        let config = Config::from_args(vec![
            "NAME",
            "--config-path",
            testing_config.to_str().unwrap(),
            "--active-color",
            "Green",
            "--window-title",
            "New window title",
            "--layout",
            "Shorter layout",
            "--todo-path",
            "todo.txt",
            "--archive-path",
            "archive.txt",
            "--file-watcher",
            "true",
            "--list-shift",
            "10",
            "--pending-sort",
            "reverse",
            "--done-sort",
            "priority",
            "--delete-final-date",
            "true",
            "--set-final-date",
            "override",
            "--preview-format",
            "extra important preview",
            "--wrap-preview",
            "true",
            "--list-refresh-rate",
            "15",
            "--list-active-color",
            "yellow ^blue",
            "--autosave-duration",
            "150",
            "--category-select-style",
            "blue",
            "--category-remove-style",
            "yellow",
        ])?;
        let mut expected = Config::default();
        expected.styles.active_color = Color::green();
        expected.ui_config.init_widget = WidgetType::Project;
        expected.ui_config.window_title = String::from("New window title");
        expected.ui_config.layout = String::from("Shorter layout");
        expected.file_worker_config.todo_path = PathBuf::from("todo.txt");
        expected.file_worker_config.archive_path = Some(PathBuf::from("archive.txt"));
        expected.file_worker_config.file_watcher = true;
        expected.list_config.list_shift = 10;
        expected.todo_config.use_done = true;
        expected.todo_config.pending_sort = TaskSort::Reverse;
        expected.todo_config.done_sort = TaskSort::Priority;
        expected.todo_config.delete_final_date = true;
        expected.todo_config.set_final_date = SetFinalDateType::Override;
        expected.preview_config.preview_format = String::from("extra important preview");
        expected.preview_config.wrap_preview = true;
        expected.ui_config.window_keybinds = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('e')), UIEvent::EditMode),
            (KeyShortcut::from(KeyCode::Char('q')), UIEvent::Quit),
            (
                KeyShortcut::new(KeyCode::Char('s'), KeyModifiers::SHIFT),
                UIEvent::Save,
            ),
            (KeyShortcut::from(KeyCode::Char('u')), UIEvent::Load),
            (
                KeyShortcut::new(KeyCode::Char('h'), KeyModifiers::SHIFT),
                UIEvent::MoveLeft,
            ),
            (
                KeyShortcut::new(KeyCode::Char('l'), KeyModifiers::SHIFT),
                UIEvent::MoveRight,
            ),
            (
                KeyShortcut::new(KeyCode::Char('k'), KeyModifiers::SHIFT),
                UIEvent::MoveUp,
            ),
            (
                KeyShortcut::new(KeyCode::Char('j'), KeyModifiers::SHIFT),
                UIEvent::MoveDown,
            ),
            (
                KeyShortcut::new(KeyCode::Char('i'), KeyModifiers::SHIFT),
                UIEvent::InsertMode,
            ),
            (
                KeyShortcut::new(KeyCode::Char('e'), KeyModifiers::SHIFT),
                UIEvent::EditMode,
            ),
            (KeyShortcut::from(KeyCode::Char('/')), UIEvent::SearchMode),
        ]);
        expected.ui_config.list_refresh_rate = Duration::from_secs(15);
        expected.active_color_config.list_active_color =
            TextStyle::default().bg(Color::blue()).fg(Color::yellow());
        expected.file_worker_config.autosave_duration = Duration::from_secs(150);
        expected.list_config.list_keybind = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('g')), UIEvent::ListLast),
            (KeyShortcut::from(KeyCode::Char('j')), UIEvent::ListDown),
            (KeyShortcut::from(KeyCode::Char('k')), UIEvent::ListUp),
            (
                KeyShortcut::new(KeyCode::Char('g'), KeyModifiers::SHIFT),
                UIEvent::ListLast,
            ),
            (KeyShortcut::from(KeyCode::Char('h')), UIEvent::CleanSearch),
        ]);
        expected.widget_base_config.tasks_keybind = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('s')), UIEvent::Select),
            (
                KeyShortcut::new(KeyCode::Char('u'), KeyModifiers::SHIFT),
                UIEvent::SwapUpItem,
            ),
            (
                KeyShortcut::new(KeyCode::Char('d'), KeyModifiers::SHIFT),
                UIEvent::SwapDownItem,
            ),
            (KeyShortcut::from(KeyCode::Char('x')), UIEvent::RemoveItem),
            (KeyShortcut::from(KeyCode::Char('d')), UIEvent::MoveItem),
            (KeyShortcut::from(KeyCode::Enter), UIEvent::Select),
            (KeyShortcut::from(KeyCode::Char('n')), UIEvent::NextSearch),
            (
                KeyShortcut::new(KeyCode::Char('n'), KeyModifiers::SHIFT),
                UIEvent::PrevSearch,
            ),
        ]);
        expected.widget_base_config.category_keybind = EventHandlerUI::from([
            (KeyShortcut::from(KeyCode::Char('r')), UIEvent::Remove),
            (KeyShortcut::from(KeyCode::Enter), UIEvent::Select),
            (KeyShortcut::from(KeyCode::Backspace), UIEvent::Remove),
            (KeyShortcut::from(KeyCode::Char('n')), UIEvent::NextSearch),
            (
                KeyShortcut::new(KeyCode::Char('n'), KeyModifiers::SHIFT),
                UIEvent::PrevSearch,
            ),
        ]);
        expected.styles.category_select_style = TextStyle::default().fg(Color::blue());
        expected.styles.category_remove_style = TextStyle::default().fg(Color::yellow());
        let mut custom_styles = CustomCategoryStyle::default();
        custom_styles.insert(
            String::from("+project"),
            TextStyle::default().fg(Color::green()),
        );
        expected.styles.custom_category_style = custom_styles;

        assert_eq!(config, expected);

        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn export_default_is_possible() -> Result<()> {
        Config::export_default(PathBuf::from("/dev/null"))?;
        Ok(())
    }
}
