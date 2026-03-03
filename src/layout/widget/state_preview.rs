use super::{widget_base::WidgetBase, widget_trait::State, widget_type::WidgetType};
use crate::{
    config::Config,
    todo::{Parser, ToDo, ToDoData},
    ui::UIEvent,
};
use anyhow::Result;
use todo_txt::Task;
use tui::{
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

pub trait Previewable: std::fmt::Debug {
    fn get_task(todo: &ToDo) -> Option<&Task>;
    fn widget_type() -> WidgetType;
}

/// Represents the state for a preview widget that displays task details.
#[derive(Debug)]
pub struct StatePreview<P: Previewable> {
    base: WidgetBase,
    parser: Parser,
    wrap_preview: bool,
    _phantom: std::marker::PhantomData<P>,
}

#[derive(Debug)]
pub struct ActivePreview;
impl Previewable for ActivePreview {
    fn get_task(todo: &ToDo) -> Option<&Task> {
        todo.get_active()
    }

    fn widget_type() -> WidgetType {
        WidgetType::Preview
    }
}

#[derive(Debug)]
pub struct PendingActualPreview;
impl Previewable for PendingActualPreview {
    fn get_task(todo: &ToDo) -> Option<&Task> {
        todo.get_actual(ToDoData::Pending)
    }

    fn widget_type() -> WidgetType {
        WidgetType::PendingLivePreview
    }
}

#[derive(Debug)]
pub struct DoneActualPreview;
impl Previewable for DoneActualPreview {
    fn get_task(todo: &ToDo) -> Option<&Task> {
        todo.get_actual(ToDoData::Done)
    }

    fn widget_type() -> WidgetType {
        WidgetType::DoneLivePreview
    }
}

impl<P: Previewable> StatePreview<P> {
    /// Creates a new `StatePreview` instance.
    ///
    /// # Parameters
    ///
    /// - `base`: The base properties shared among different widget types.
    /// - `format`: The format string used to generate the content for the preview.
    ///
    /// # Returns
    ///
    /// A new `StatePreview` instance.
    pub fn new(base: WidgetBase, config: &Config) -> Result<Self> {
        Ok(Self {
            base,
            parser: Parser::new(&config.preview_config.preview_format, config.styles.clone())?,
            wrap_preview: config.preview_config.wrap_preview,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<P: Previewable> State for StatePreview<P> {
    fn handle_event_state(&mut self, _: UIEvent, _todo: &mut ToDo) -> bool {
        false
    }

    fn render(&self, f: &mut Frame, todo: &ToDo) {
        let lines = match P::get_task(todo) {
            Some(act_task) => self.parser.fill(act_task, todo),
            None => vec![],
        };
        let mut paragraph = Paragraph::new(
            lines
                .iter()
                .map(|line| Line {
                    spans: line
                        .iter()
                        .map(|(text, style)| Span::styled(text, *style))
                        .collect::<Vec<_>>(),
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
        )
        .block(self.get_block());
        if self.wrap_preview {
            paragraph = paragraph.wrap(Wrap { trim: true })
        }
        f.render_widget(paragraph, self.base.chunk);
    }

    fn get_base(&self) -> &WidgetBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut WidgetBase {
        &mut self.base
    }

    fn focus_event(&mut self, _: &ToDo) -> bool {
        false
    }

    fn widget_type(&self) -> WidgetType {
        P::widget_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{layout::Render, todo::ToDoData};
    use std::str::FromStr;
    use test_log::test;
    use todo_txt::Task;
    use tui::{backend::TestBackend, prelude::Rect, Terminal};

    fn make_preview<P: Previewable>(title: &str, format: &str) -> (StatePreview<P>, Config) {
        let mut config = Config::default();
        config.preview_config.preview_format = String::from(format);
        let base = WidgetBase::new(title, &config);
        let preview = StatePreview::new(base, &config).unwrap();
        (preview, config)
    }

    #[test]
    fn render_empty_when_no_active_task() -> Result<()> {
        let (mut preview, _) = make_preview::<ActivePreview>("preview", "Subject: $subject");
        let area = Rect::new(0, 0, 30, 5);
        preview.update_chunk(area);

        let todo = ToDo::default();

        let backend = TestBackend::new(30, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| {
            State::render(&preview, f, &todo);
        })?;

        terminal.backend().assert_buffer_lines([
            "╭preview─────────────────────╮",
            "│                            │",
            "│                            │",
            "│                            │",
            "╰────────────────────────────╯",
        ]);
        Ok(())
    }

    #[test]
    fn render_active_task_preview() -> Result<()> {
        let (mut preview, _) = make_preview::<ActivePreview>("preview", "Subject: $subject");
        let area = Rect::new(0, 0, 30, 5);
        preview.update_chunk(area);

        let mut todo = ToDo::default();
        todo.new_task("Buy groceries")?;
        todo.set_active(ToDoData::Pending, 0);

        let backend = TestBackend::new(30, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| {
            State::render(&preview, f, &todo);
        })?;

        terminal.backend().assert_buffer_lines([
            "╭preview─────────────────────╮",
            "│Subject: Buy groceries      │",
            "│                            │",
            "│                            │",
            "╰────────────────────────────╯",
        ]);
        Ok(())
    }

    #[test]
    fn render_with_counts() -> Result<()> {
        let (mut preview, _) =
            make_preview::<ActivePreview>("preview", "Pending: $pending Done: $done");
        let area = Rect::new(0, 0, 34, 5);
        preview.update_chunk(area);

        let mut todo = ToDo::default();
        todo.new_task("task one")?;
        todo.new_task("task two")?;
        todo.new_task("x done task")?;
        todo.set_active(ToDoData::Pending, 0);

        let backend = TestBackend::new(34, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| {
            State::render(&preview, f, &todo);
        })?;

        terminal.backend().assert_buffer_lines([
            "╭preview─────────────────────────╮",
            "│Pending: 2 Done: 1              │",
            "│                                │",
            "│                                │",
            "╰────────────────────────────────╯",
        ]);
        Ok(())
    }

    #[test]
    fn pending_live_preview_shows_actual_task() -> Result<()> {
        let (mut preview, _) =
            make_preview::<PendingActualPreview>("pending live preview", "Subject: $subject");
        let area = Rect::new(0, 0, 40, 5);
        preview.update_chunk(area);

        let mut todo = ToDo::default();
        todo.new_task("first task")?;
        todo.new_task("second task")?;
        todo.set_actual(ToDoData::Pending, 1);

        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| {
            State::render(&preview, f, &todo);
        })?;

        terminal.backend().assert_buffer_lines([
            "╭pending live preview──────────────────╮",
            "│Subject: second task                  │",
            "│                                      │",
            "│                                      │",
            "╰──────────────────────────────────────╯",
        ]);
        Ok(())
    }

    #[test]
    fn pending_live_preview_shows_first_when_not_set() -> Result<()> {
        let (mut preview, _) =
            make_preview::<PendingActualPreview>("pending live preview", "Subject: $subject");
        let area = Rect::new(0, 0, 40, 5);
        preview.update_chunk(area);

        let mut todo = ToDo::default();
        todo.new_task("first task")?;
        todo.new_task("second task")?;

        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| {
            State::render(&preview, f, &todo);
        })?;

        terminal.backend().assert_buffer_lines([
            "╭pending live preview──────────────────╮",
            "│Subject: first task                   │",
            "│                                      │",
            "│                                      │",
            "╰──────────────────────────────────────╯",
        ]);
        Ok(())
    }

    #[test]
    fn done_live_preview_shows_actual_done_task() -> Result<()> {
        let (mut preview, _) =
            make_preview::<DoneActualPreview>("done live preview", "Subject: $subject");
        let area = Rect::new(0, 0, 40, 5);
        preview.update_chunk(area);

        let mut todo = ToDo::default();
        let mut task1 = Task::from_str("done first").unwrap();
        task1.finished = true;
        todo.add_task(task1);
        let mut task2 = Task::from_str("done second").unwrap();
        task2.finished = true;
        todo.add_task(task2);
        todo.set_actual(ToDoData::Done, 1);

        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| {
            State::render(&preview, f, &todo);
        })?;

        terminal.backend().assert_buffer_lines([
            "╭done live preview─────────────────────╮",
            "│Subject: done second                  │",
            "│                                      │",
            "│                                      │",
            "╰──────────────────────────────────────╯",
        ]);
        Ok(())
    }

    #[test]
    fn pending_live_preview_empty_when_no_tasks() -> Result<()> {
        let (mut preview, _) =
            make_preview::<PendingActualPreview>("pending live preview", "Subject: $subject");
        let area = Rect::new(0, 0, 40, 5);
        preview.update_chunk(area);

        let todo = ToDo::default();

        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| {
            State::render(&preview, f, &todo);
        })?;

        terminal.backend().assert_buffer_lines([
            "╭pending live preview──────────────────╮",
            "│                                      │",
            "│                                      │",
            "│                                      │",
            "╰──────────────────────────────────────╯",
        ]);
        Ok(())
    }

    #[test]
    fn pending_live_preview_widget_type() {
        let (preview, _) = make_preview::<PendingActualPreview>("pending live preview", "$subject");
        assert_eq!(preview.widget_type(), WidgetType::PendingLivePreview);
    }

    #[test]
    fn done_live_preview_widget_type() {
        let (preview, _) = make_preview::<DoneActualPreview>("done live preview", "$subject");
        assert_eq!(preview.widget_type(), WidgetType::DoneLivePreview);
    }

    #[test]
    fn active_preview_widget_type() {
        let (preview, _) = make_preview::<ActivePreview>("preview", "$subject");
        assert_eq!(preview.widget_type(), WidgetType::Preview);
    }
}
