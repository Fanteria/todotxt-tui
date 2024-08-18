mod common;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::KeyCode;
use todotxt_tui::config::Color;
use todotxt_tui::config::Conf;
use todotxt_tui::config::ConfMerge;
use todotxt_tui::config::Config;

use pretty_assertions::assert_eq;
use test_log::test;
use todotxt_tui::config::CustomCategoryStyle;
use todotxt_tui::config::SetFinalDateType;
use todotxt_tui::config::TaskSort;
use todotxt_tui::config::TextStyle;
use todotxt_tui::layout::widget::widget_type::WidgetType;
use todotxt_tui::ui::EventHandlerUI;
use todotxt_tui::ui::UIEvent;
use todotxt_tui::ToDoRes;

#[test]
fn empty_config() -> ToDoRes<()> {
    let empty_config = common::get_test_file("empty_config.toml");
    let default = Config::from_file(&empty_config)?;
    assert_eq!(default, Config::default());

    Ok(())
}

#[test]
fn changed_config() -> ToDoRes<()> {
    let testing_config = common::get_test_file("testing_config.toml");
    let config = Config::from_file(&testing_config)?;
    let mut expected = Config::default();
    expected.styles.active_color = Color::blue();
    expected.ui_config.init_widget = WidgetType::Project;
    expected.ui_config.window_title = String::from("Window title");
    expected.ui_config.layout = String::from("Short invalid layout");
    expected.file_worker_config.todo_path = PathBuf::from("invalid/path/to/todo.txt");
    expected.file_worker_config.archive_path = Some(PathBuf::from("invalid/path/to/archive.txt"));
    expected.file_worker_config.file_watcher = false;
    expected.list_config.list_shift = 0;
    expected.todo_config.use_done = true;
    expected.todo_config.pending_sort = TaskSort::Priority;
    expected.todo_config.done_sort = TaskSort::Reverse;
    expected.todo_config.delete_final_date = false;
    expected.todo_config.set_final_date = SetFinalDateType::Never;
    expected.preview_config.preview_format = String::from("unimportant preview");
    expected.preview_config.wrap_preview = false;
    expected.ui_config.window_keybinds =
        EventHandlerUI::new(&[(KeyCode::Char('e'), UIEvent::EditMode)]);
    expected.ui_config.list_refresh_rate = Duration::from_secs(10);
    expected.active_color_config.list_active_color = TextStyle::default().bg(Color::green());
    expected.file_worker_config.autosave_duration = Duration::from_secs(100);
    expected.list_config.list_keybind =
        EventHandlerUI::new(&[(KeyCode::Char('g'), UIEvent::ListLast)]);
    expected.widget_base_config.tasks_keybind =
        EventHandlerUI::new(&[(KeyCode::Char('s'), UIEvent::Select)]);
    expected.widget_base_config.category_keybind =
        EventHandlerUI::new(&[(KeyCode::Char('r'), UIEvent::Remove)]);
    expected.styles.category_select_style = TextStyle::default().fg(Color::red());
    expected.styles.category_remove_style = TextStyle::default().fg(Color::green());
    expected.styles.custom_category_style = CustomCategoryStyle::default();
    expected.styles.custom_category_style.insert(
        String::from("+project"),
        TextStyle::default().fg(Color::green()),
    );

    assert_eq!(config, expected);

    Ok(())
}

#[test]
fn default_values_clap() -> ToDoRes<()> {
    let empty_config = common::get_test_file("empty_config.toml");
    let default = Config::from_args(vec![
        "NAME",
        "--config-path",
        empty_config.to_str().unwrap(),
    ])?;
    assert_eq!(default, Config::default());
    Ok(())
}

#[test]
fn custom_clap_arguments() -> ToDoRes<()> {
    let testing_config = common::get_test_file("testing_config.toml");
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
        "--list-shift",
        "10",
        "--pending-sort",
        "reverse",
        "--done-sort",
        "priority",
        "--delete-final-date",
        "--set-final-date",
        "override",
        "--preview-format",
        "extra important preview",
        "--wrap-preview",
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
    expected.ui_config.window_keybinds =
        EventHandlerUI::new(&[(KeyCode::Char('e'), UIEvent::EditMode)]);
    expected.ui_config.list_refresh_rate = Duration::from_secs(15);
    expected.active_color_config.list_active_color =
        TextStyle::default().bg(Color::blue()).fg(Color::yellow());
    expected.file_worker_config.autosave_duration = Duration::from_secs(150);
    expected.list_config.list_keybind =
        EventHandlerUI::new(&[(KeyCode::Char('g'), UIEvent::ListLast)]);
    expected.widget_base_config.tasks_keybind =
        EventHandlerUI::new(&[(KeyCode::Char('s'), UIEvent::Select)]);
    expected.widget_base_config.category_keybind =
        EventHandlerUI::new(&[(KeyCode::Char('r'), UIEvent::Remove)]);
    expected.styles.category_select_style = TextStyle::default().fg(Color::blue());
    expected.styles.category_remove_style = TextStyle::default().fg(Color::yellow());
    let mut custom_styles = HashMap::new();
    custom_styles.insert("+project", TextStyle::default().fg(Color::green()));

    assert_eq!(config, expected);

    Ok(())
}
