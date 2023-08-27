use super::{widget_base::WidgetBase, widget_trait::State};
use crate::{todo::ToDoData, CONFIG, ui::UIEvent};
use chrono::NaiveDate;
use tui::{
    backend::Backend,
    widgets::{Paragraph, Wrap},
    Frame,
};

pub struct StatePreview {
    base: WidgetBase,
    format: String,
}

impl StatePreview {
    pub fn new(base: WidgetBase, format: String) -> Self {
        StatePreview { format, base }
    }

    fn get_content(&self) -> String {
        let data = self.data();
        let task = match data.get_active() {
            Some(s) => s,
            None => return String::from(""),
        };
        let date_to_str = |date: Option<NaiveDate>| match date {
            Some(date) => date.to_string(),
            None => String::from(""),
        };
        self.format
            .replace("{n}", &data.len(ToDoData::Pending).to_string())
            .replace("{N}", &data.len(ToDoData::Done).to_string())
            .replace("{s}", &task.subject)
            .replace("{p}", &task.priority.to_string())
            .replace("{c}", &date_to_str(task.create_date))
            .replace("{f}", &date_to_str(task.finish_date))
            .replace("{F}", &task.finished.to_string())
            .replace("{t}", &date_to_str(task.threshold_date))
            .replace("{d}", &date_to_str(task.due_date))
            .replace("{C}", &task.contexts().join(", "))
            .replace("{P}", &task.projects().join(", "))
            .replace("{H}", &task.hashtags.join(", "))
    }
}

impl State for StatePreview {
    fn handle_event_state(&mut self, _: UIEvent) -> bool {
        false
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let mut paragraph = Paragraph::new(self.get_content()).block(self.get_block());
        if CONFIG.wrap_preview {
            paragraph = paragraph.wrap(Wrap { trim: true })
        }
        // .style(Style::default().fg(Color::White).bg(Color::Black));
        // .alignment(Alignment::Center)
        f.render_widget(paragraph, self.base.chunk);
    }

    fn get_base(&self) -> &WidgetBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut WidgetBase {
        &mut self.base
    }
}
