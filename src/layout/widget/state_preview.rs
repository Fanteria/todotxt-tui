use super::{widget_base::WidgetBase, widget_trait::State};
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
    fn handle_event_state(&mut self, _: UIEvent) -> bool {
        false
    }

    fn render(&self, f: &mut Frame) {
        let todo: &ToDo = &self.base.data();
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

    fn focus_event(&mut self) -> bool {
        false
    }
}
