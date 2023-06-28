pub mod container;
pub mod widget;

use self::{
    container::{Container, InitItem, Item, RcCon},
    widget::widget_type::WidgetType,
    widget::Widget,
};
use crate::{error::ToDoRes, todo::ToDo};
use crossterm::event::KeyEvent;
use std::cell::RefCell;
use std::rc::Rc;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Direction::Horizontal, Direction::Vertical, Rect},
    Frame,
};

pub struct Layout {
    root: Rc<RefCell<Container>>,
    actual: Rc<RefCell<Container>>,
}

impl Layout {
    pub fn new(chunk: Rect, actual: WidgetType, data: Rc<RefCell<ToDo>>) -> Layout {
        let input_widget = Widget::new(WidgetType::Input, "Input", data.clone());
        let list_widget = Widget::new(WidgetType::List, "List", data.clone());
        let done_widget = Widget::new(WidgetType::Done, "Done", data.clone());
        let categories_widget = Widget::new(WidgetType::Project, "Projects", data.clone());

        let root = Container::new(
            vec![
                InitItem::InitWidget(input_widget),
                InitItem::InitContainer(Container::new(
                    vec![
                        InitItem::InitWidget(list_widget),
                        InitItem::InitContainer(Container::new(
                            vec![
                                InitItem::InitWidget(done_widget),
                                InitItem::InitWidget(categories_widget),
                            ],
                            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
                            Vertical,
                            None,
                        )),
                    ],
                    vec![Constraint::Percentage(50), Constraint::Percentage(50)],
                    Horizontal,
                    None,
                )),
            ],
            vec![Constraint::Length(3), Constraint::Percentage(30)],
            Vertical,
            None,
        );
        let actual = Container::select_widget(root.clone(), actual).unwrap(); // TODO
        root.borrow_mut().update_chunks(chunk);
        if let Item::Widget(w) = actual.borrow_mut().actual_item_mut() {
            w.widget.focus();
        }

        Layout { root, actual }
    }

    fn value_from_string(value: &str) -> Constraint {
        if value.is_empty() {
            return Constraint::Percentage(50);
        }

        match value.find('%') {
            Some(i) => {
                let x = &value[..i];
                Constraint::Percentage(10)
            }
            None => {
                Constraint::Length(10)
            }
        }
    }

    fn from_str(template: &str, chunk: Rect, actual: WidgetType, data: Rc<RefCell<ToDo>>) {
        // Find first '[' and move start of template to it (start of first container)
        let index = match template.find('[') {
            Some(i) => i,
            None => return,
        };
        let template = &template[index + 1..];

        // Define separators
        const ITEM_SEPARATOR: char = ',';
        const ARG_SEPARATOR: char = ':';
        const START_CONTAINER: char = '[';
        const END_CONTAINER: char = ']';

        let mut string = String::new();
        let mut item = String::new();
        let mut indent = 0;

        let mut container: Vec<(Direction, Constraint)> = Vec::new();
        container.push((Direction::Vertical, Constraint::Percentage(50)));

        for (i, ch) in template.chars().enumerate() {
            match ch {
                START_CONTAINER => {
                    let new_direction = match container.last().unwrap().0 {
                        Vertical => Horizontal,
                        Horizontal => Vertical,
                    };
                    container.push((new_direction, Constraint::Percentage(50)));

                    println!("{}[{}", " ".repeat(indent), string);
                    indent += 1;
                    string.clear();
                }
                END_CONTAINER => {
                    let cont = container.pop().unwrap();
                    // End of the brackets stack, end cycle
                    if container.is_empty() {
                        break;
                    }
                    indent -= 1;
                    println!("{}{}]", " ".repeat(indent), string);
                    string.clear();
                }
                ARG_SEPARATOR => {
                    item = string;
                    string = String::new();
                }
                ITEM_SEPARATOR => {
                    // Skip leading ITEM_SEPARATOR
                    if string.is_empty() {
                        continue;
                    }
                    if item.is_empty() {
                        item = string.to_lowercase();
                        string.clear();
                    } else {
                        item = item.to_lowercase();
                        string = string.to_lowercase();
                    }
                    println!("{}Item: {}, arg: {}", " ".repeat(indent), item, string);
                    match item.as_str() {
                        "direction" => match string.as_str() {
                            "" => {}
                            "vertical" => {}
                            "horizontal" => {}
                            _ => {}
                        },
                        "size" => {
                            let (x, _) = container.last().unwrap();
                        }
                        "input" => {}
                        "list" => {}
                        "done" => {}
                        "projects" => {}
                        "contexts" => {}
                        "hashtags" => {}
                        _ => {}
                    }
                    item.clear();
                    string.clear();
                }
                ' ' => {}
                '\n' => {}
                _ => string.push(ch),
            }
        }
    }

    fn move_focus(
        container: RcCon,
        direction: &Direction,
        f: fn(RcCon) -> Option<RcCon>,
    ) -> Option<RcCon> {
        let move_to_parent = || {
            let mut c = container.borrow_mut();
            if let Some(parent) = &c.parent {
                return Layout::move_focus(parent.clone(), direction, f).map(|ret| {
                    c.active = false;
                    ret
                });
            }
            None
        };

        if container.borrow().direction == *direction {
            return f(container.clone()).or_else(move_to_parent);
        }

        move_to_parent()
    }

    fn change_focus(&mut self, next: Option<RcCon>) {
        let next = match next {
            Some(s) => s,
            None => return,
        };
        self.actual.borrow_mut().unfocus();
        next.borrow_mut().focus();
        self.actual = next;
    }

    pub fn left(&mut self) {
        self.change_focus(Self::move_focus(
            Rc::clone(&self.actual),
            &Horizontal,
            Container::previous_item,
        ));
    }

    pub fn right(&mut self) {
        self.change_focus(Self::move_focus(
            Rc::clone(&self.actual),
            &Horizontal,
            Container::next_item,
        ));
    }

    pub fn up(&mut self) {
        self.change_focus(Self::move_focus(
            Rc::clone(&self.actual),
            &Vertical,
            Container::previous_item,
        ));
    }

    pub fn down(&mut self) {
        self.change_focus(Self::move_focus(
            Rc::clone(&self.actual),
            &Vertical,
            Container::next_item,
        ));
    }

    pub fn cursor_visible(&self) -> bool {
        match &self.actual.borrow().actual_item() {
            Item::Widget(w) => w.widget.cursor_visible(),
            Item::Container(_) => false, // TODO return Error
        }
    }

    pub fn select_widget(&mut self, widget_type: WidgetType) -> ToDoRes<()> {
        self.actual = Container::select_widget(self.root.clone(), widget_type)?;
        if let Item::Widget(w) = self.actual.borrow_mut().actual_item_mut() {
            w.widget.focus();
        }
        Ok(())
    }

    pub fn handle_key(&self, event: &KeyEvent) {
        let mut act = self.actual.borrow_mut();
        match &mut act.actual_item_mut() {
            Item::Widget(w) => w.widget.handle_key(event),
            Item::Container(_) => {} // TODO return Error
        }
    }

    pub fn update_chunks(&mut self, chunk: Rect) {
        self.root.borrow_mut().update_chunks(chunk);
    }

    pub fn render<B>(&self, f: &mut Frame<B>)
    where
        B: Backend,
    {
        self.root.borrow().render_recursive(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_layout() -> Layout {
        Layout::new(
            Rect::new(0, 0, 0, 0),
            WidgetType::List,
            Rc::new(RefCell::new(ToDo::new(false))),
        )
    }

    #[test]
    fn test_basic_movement() -> ToDoRes<()> {
        let mut l = mock_layout();
        let check_type = |widget_type, l: &Layout| -> ToDoRes<()> {
            let active = l.actual.as_ref().borrow().get_active_type()?;
            if active != widget_type {
                panic!("Active widget must be {:?} not {:?}.", widget_type, active)
            }
            Ok(())
        };

        l.right();
        check_type(WidgetType::Done, &l)?;

        l.right();
        check_type(WidgetType::Done, &l)?;

        l.down();
        check_type(WidgetType::Project, &l)?;

        l.right();
        check_type(WidgetType::Project, &l)?;

        l.down();
        check_type(WidgetType::Project, &l)?;

        l.left();
        check_type(WidgetType::List, &l)?;

        l.right();
        check_type(WidgetType::Project, &l)?;

        l.left();
        check_type(WidgetType::List, &l)?;

        l.up();
        check_type(WidgetType::Input, &l)?;

        Ok(())
    }

    #[test]
    fn test_from_string() {
        let str_layout = r#"
            [
              Direction: Vertical,
              Input: 3,
              [
                Direction:Horizontal,
                Size: 50%,
                List: 50%,
                [ dIrEcTiOn: VeRtIcAl,
                  Done,
                  Hashtags: 50%,
                ],
                Projects: 50%,
              ],
            ]
            
            Direction: ERROR,
        "#;

        Layout::from_str(
            str_layout,
            Rect::new(0, 0, 0, 0),
            WidgetType::List,
            Rc::new(RefCell::new(ToDo::new(false))),
        );
    }
}
