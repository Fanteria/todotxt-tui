use super::{widget_trait::State, Widget};
use crate::{
    todo::{ToDo, ToDoData},
    utils::get_block,
};
use chrono::NaiveDate;
use crossterm::event::KeyEvent;
use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};
use std::sync::{Arc, Mutex};

pub struct StatePreview {
    data: Arc<Mutex<ToDo>>,
    format: String,
    focus: bool,
}

impl StatePreview {
    pub fn new(format: &str, data: Arc<Mutex<ToDo>>) -> Self {
        StatePreview {
            data,
            format: String::from(format),
            focus: false,
        }
    }

    fn get_content(&self) -> String {
        let data = self.data.lock().unwrap();
        let task = match data.get_active() {
            Some(s) => s,
            None => return String::from(""),
        };
        let date_to_str = |date: Option<NaiveDate>| {
            match date {
                Some(date) => date.to_string(),
                None => String::from(""),
            }
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
    fn handle_key(&mut self, event: &KeyEvent) {}

    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget) {
        let paragraph = Paragraph::new(self.get_content()).block(get_block("Title", self.focus));
        // .style(Style::default().fg(Color::White).bg(Color::Black));
        // .alignment(Alignment::Center)
        // .wrap(Wrap { trim: true });
        f.render_widget(paragraph, widget.chunk);
    }

    fn focus(&mut self) {
        self.focus = true;
    }

    fn unfocus(&mut self) {
        self.focus = false;
    }

    fn cursor_visible(&self) -> bool {
        false
    }
}
