use super::{widget_trait::State, Widget};
use crate::{
    todo::{ToDo, ToDoData},
    utils::get_block,
};
use chrono::NaiveDate;
use crossterm::event::KeyEvent;
use std::cell::RefCell;
use std::rc::Rc;
use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

pub struct StatePreview {
    data: Rc<RefCell<ToDo>>,
    format: String,
    focus: bool,
}

impl StatePreview {
    pub fn new(format: &str, data: Rc<RefCell<ToDo>>) -> Self {
        StatePreview {
            data,
            format: String::from(format),
            focus: false,
        }
    }

    fn get_content(&self) -> String {
        let borrowed = self.data.borrow();
        let task = match borrowed.get_active() {
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
            .replace("{n}", &self.data.borrow().len(ToDoData::Pending).to_string())
            .replace("{N}", &self.data.borrow().len(ToDoData::Done).to_string())
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
