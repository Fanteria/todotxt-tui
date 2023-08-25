use super::widget_trait::State;
use crate::{
    todo::{ToDo, ToDoData},
    CONFIG,
};
use chrono::NaiveDate;
use crossterm::event::KeyEvent;
use std::sync::{Arc, Mutex};
use tui::{
    backend::Backend,
    prelude::Rect,
    widgets::{Paragraph, Wrap},
    Frame,
};

pub struct StatePreview {
    data: Arc<Mutex<ToDo>>,
    format: String,
    focus: bool,
    chunk: Rect,
    title: String,
}

impl StatePreview {
    pub fn new(format: &str, data: Arc<Mutex<ToDo>>, title: &str) -> Self {
        StatePreview {
            data,
            format: String::from(format),
            focus: false,
            chunk: Rect::default(),
            title: String::from(title),
        }
    }

    fn get_content(&self) -> String {
        let data = self.data.lock().unwrap();
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
    fn handle_key(&mut self, _: &KeyEvent) {}

    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let mut paragraph = Paragraph::new(self.get_content()).block(self.get_block());
        if CONFIG.wrap_preview {
            paragraph = paragraph.wrap(Wrap { trim: true })
        }
        // .style(Style::default().fg(Color::White).bg(Color::Black));
        // .alignment(Alignment::Center)
        f.render_widget(paragraph, self.chunk);
    }

    fn update_chunk(&mut self, chunk: Rect) {
        self.chunk = chunk;
    }

    fn get_focus_mut(&mut self) -> &mut bool {
        &mut self.focus
    }

    fn get_focus(&self) -> bool {
        self.focus
    }
        
    fn get_title(&self) -> &str {
        &self.title
    }
}
