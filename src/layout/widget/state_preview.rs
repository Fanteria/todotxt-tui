use super::{widget_base::WidgetBase, widget_trait::State, widget_type::WidgetType};
use crate::{
    config::Config,
    todo::{Parser, ToDo},
    ui::UIEvent,
    Result,
};
use tui::{
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

/// Represents the state for a preview widget that displays task details.
#[derive(Debug)]
pub struct StatePreview {
    base: WidgetBase,
    parser: Parser,
    wrap_preview: bool,
}

impl StatePreview {
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
        Ok(StatePreview {
            base,
            parser: Parser::new(&config.preview_config.preview_format, config.styles.clone())?,
            wrap_preview: config.preview_config.wrap_preview,
        })
    }
}

impl State for StatePreview {
    fn handle_event_state(&mut self, _: UIEvent, _todo: &mut ToDo) -> bool {
        false
    }

    fn render(&self, f: &mut Frame, todo: &ToDo) {
        let lines = match todo.get_active() {
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
        WidgetType::Preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{layout::Render, todo::ToDoData};
    use test_log::test;
    use tui::{backend::TestBackend, prelude::Rect, Terminal};

    fn make_preview(format: &str) -> (StatePreview, Config) {
        let mut config = Config::default();
        config.preview_config.preview_format = String::from(format);
        let base = WidgetBase::new(&WidgetType::Preview, &config);
        let preview = StatePreview::new(base, &config).unwrap();
        (preview, config)
    }

    #[test]
    fn render_empty_when_no_active_task() -> Result<()> {
        let (mut preview, _) = make_preview("Subject: $subject");
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
        let (mut preview, _) = make_preview("Subject: $subject");
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
        let (mut preview, _) = make_preview("Pending: $pending Done: $done");
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
}
