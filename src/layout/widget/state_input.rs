use super::{widget_trait::State, Widget};
use crate::{
    todo::ToDo,
    utils::{get_block, some_or_return},
};
use crossterm::event::{KeyCode, KeyEvent};
use std::cell::RefCell;
use std::rc::Rc;
use tui::{backend::Backend, widgets::Paragraph, Frame};

type RCToDo = Rc<RefCell<ToDo>>;

pub struct StateInput {
    actual: String,
    data: RCToDo,
}

impl StateInput {
    pub fn new(data: RCToDo) -> Self {
        Self {
            actual: String::from(""),
            data,
        }
    }

    fn autocomplete(&mut self) {
        let last_space_index = self
            .actual
            .rfind(' ')
            .and_then(|i| Some(i + 1))
            .unwrap_or(0);
        let base = some_or_return!(self.actual.get(last_space_index..));
        let category = some_or_return!(base.get(0..1));
        let pattern = some_or_return!(base.get(1..));

        let data = self.data.borrow();
        let list =  match category {
            "+" => data.get_projects(),
            "@" => data.get_contexts(),
            "#" => data.get_hashtags(),
            _ => return,
        };

        if list.is_empty() {
            return;
        }

        let list = list.start_with(pattern);

        let same_start_index = |fst: &str, sec: &str| -> usize {
            for (i, (fst_char, sec_char)) in fst
                .chars()
                .into_iter()
                .zip(sec.chars().into_iter())
                .enumerate()
            {
                if fst_char != sec_char {
                    return i;
                }
            }
            std::cmp::min(fst.len(), sec.len())
        };
        if list.is_empty() {
            return;
        }

        let mut new_act = list[0].as_str();

        if list.len() != 1 {
            list.iter()
                .skip(1)
                .for_each(|item| new_act = &new_act[..same_start_index(new_act, item)]);
            self.actual += &new_act[pattern.len()..];
        } else {
            self.actual += &new_act[pattern.len()..];
            self.actual += " ";
        }
    }
}

impl State for StateInput {
    fn handle_key(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char(ch) => self.actual.push(ch),
            KeyCode::Backspace => {
                self.actual.pop();
            }
            KeyCode::Enter => match self.data.borrow_mut().new_task(&self.actual) {
                Ok(_) => self.actual.clear(),
                Err(_) => self.actual += " wrong!!!",
            },
            KeyCode::Esc => self.actual.clear(),
            KeyCode::Tab => self.autocomplete(),
            _ => {}
        }
    }

    fn render<B: Backend>(&self, f: &mut Frame<B>, active: bool, widget: &Widget) {
        f.render_widget(
            Paragraph::new(self.actual.clone()).block(get_block(&widget.title, active)),
            widget.chunk,
        );
    }

    fn focus(&mut self) {}

    fn unfocus(&mut self) {}

    fn cursor_visible(&self) -> bool {
        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_autocomplete() -> Result<(), Box<dyn Error>> {
        // prepare testing
        let testing_string: &str = r#"
        1 +project1 @context1 #hashtag1 
        2 +project2 @context2
        3 +project3 @context3
        4 +name_project2 @context3 #hashtag1
        5 +name_project3 @context3 #hashtag2
        6 +unique @context2 #hashtag2
        "#;
        let mut widget = StateInput::new(Rc::new(RefCell::new(ToDo::load(
            testing_string.as_bytes(),
            false,
        )?)));

        // not found check
        widget.actual = String::from("some text +missing");
        widget.autocomplete();
        assert_eq!(widget.actual, "some text +missing");

        // group check
        widget.actual = String::from("some text +pr");
        widget.autocomplete();
        assert_eq!(widget.actual, "some text +project");

        // double group check
        widget.actual = String::from("some text +project1 +name");
        widget.autocomplete();
        assert_eq!(widget.actual, "some text +project1 +name_project");

        // unique check
        widget.actual = String::from("text +uni");
        widget.autocomplete();
        assert_eq!(widget.actual, "text +unique ");

        // empty task description check
        widget.actual = String::from("+uni");
        widget.autocomplete();
        assert_eq!(widget.actual, "+unique ");

        // context check
        widget.actual = String::from("@con");
        widget.autocomplete();
        assert_eq!(widget.actual, "@context");

        widget.actual = String::from("@context1");
        widget.autocomplete();
        assert_eq!(widget.actual, "@context1 ");

        // hashtag check
        widget.actual = String::from("#hash");
        widget.autocomplete();
        assert_eq!(widget.actual, "#hashtag");

        widget.actual = String::from("#hashtag2");
        widget.autocomplete();
        assert_eq!(widget.actual, "#hashtag2 ");

        Ok(())
    }
}
