pub mod container;
mod render_trait;
pub mod widget;

use self::{
    container::{Container, Item, RcCon},
    widget::{widget_type::WidgetType, Widget},
};
use crate::layout::widget::widget_trait::State;
use crate::{
    error::{ToDoError, ToDoRes},
    todo::ToDo,
    CONFIG,
};
use crossterm::event::KeyEvent;
pub use render_trait::Render;
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
    {cell::RefCell, str::FromStr},
};
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
    fn value_from_string(value: &str) -> ToDoRes<Constraint> {
        if value.is_empty() {
            return Ok(Constraint::Percentage(50));
        }

        match value.find('%') {
            Some(i) => {
                if i + 1 < value.len() {
                    Err(ToDoError::ParseUnknownValue)
                } else {
                    Ok(Constraint::Percentage(value[..i].parse()?))
                }
            }
            None => Ok(Constraint::Length(value.parse()?)),
        }
    }

    pub fn from_str(template: &str, data: Arc<Mutex<ToDo>>) -> ToDoRes<Self> {
        // Find first '[' and move start of template to it (start of first container)
        let index = match template.find('[') {
            Some(i) => i,
            None => return Err(ToDoError::ParseNotStart),
        };
        let template = &template[index + 1..];

        // Define separators
        const ITEM_SEPARATOR: char = ',';
        const ARG_SEPARATOR: char = ':';
        const START_CONTAINER: char = '[';
        const END_CONTAINER: char = ']';

        let mut string = String::new();
        let mut item = String::new();

        let mut container: Vec<(Direction, Constraint, Vec<Item>, Vec<Constraint>)> = Vec::new();
        container.push((
            Direction::Vertical,
            Constraint::Percentage(50),
            Vec::new(),
            Vec::new(),
        ));

        for ch in template.chars() {
            match ch {
                START_CONTAINER => {
                    let new_direction = match container.last().unwrap().0 {
                        Vertical => Horizontal,
                        Horizontal => Vertical,
                    };
                    container.push((
                        new_direction,
                        Constraint::Percentage(50),
                        Vec::new(),
                        Vec::new(),
                    ));

                    string.clear();
                }
                END_CONTAINER => {
                    let cont = container.pop().unwrap();
                    // End of the brackets stack, end cycle
                    if container.is_empty() {
                        let root = Container::new(cont.2, cont.3, cont.0, None);
                        let actual =
                            Container::select_widget(root.clone(), CONFIG.init_widget).unwrap();
                        actual.borrow_mut().actual_mut()?.focus();
                        // if let IItem::Widget(w) = actual.borrow_mut().actual_item_mut() {
                        //     w.widget.focus();
                        // } TODO remove comment
                        return Ok(Layout { root, actual });
                    }
                    let c = Item::Container(Container::new(cont.2, cont.3, cont.0, None));
                    container.last_mut().unwrap().2.push(c);
                    container.last_mut().unwrap().3.push(cont.1);
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
                    match item.as_str() {
                        "direction" => match string.as_str() {
                            "" | "vertical" => {
                                let direction = &mut container.last_mut().unwrap().0;
                                *direction = Direction::Vertical;
                            }
                            "horizontal" => {
                                let direction = &mut container.last_mut().unwrap().0;
                                *direction = Direction::Horizontal;
                            }
                            _ => return Err(ToDoError::ParseInvalidDirection(string)),
                        },
                        "size" => {
                            container.last_mut().unwrap().1 = Self::value_from_string(&string)?;
                        }
                        _ => {
                            let widget_type = WidgetType::from_str(&item)?;
                            let cont = container.last_mut().unwrap();
                            cont.2
                                .push(Item::Widget(Widget::new(widget_type, data.clone())));
                            cont.3.push(Self::value_from_string(&string)?);
                        }
                    }
                    item.clear();
                    string.clear();
                }
                ' ' => {}
                '\n' => {}
                _ => string.push(ch),
            };
        }
        Err(ToDoError::ParseNotEnd)
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
                    c.unfocus();
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

    pub fn handle_key(&self, event: &KeyEvent) {
        self.actual
            .borrow_mut()
            .actual_mut()
            .unwrap() // TODO remove
            .handle_key(event);
    }
}

impl Render for Layout {
    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        self.root.borrow().render(f);
    }

    fn unfocus(&mut self) {
        self.actual.borrow_mut().unfocus();
    }

    fn focus(&mut self) {
        self.actual.borrow_mut().focus();
    }

    fn update_chunk(&mut self, chunk: Rect) {
        self.root.borrow_mut().update_chunk(chunk);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_layout() -> Layout {
        let mock_layout = r#"
        [
            Direction: Horizontal,
            Size: 50%,
            [
                List: 50%,
                Preview,
            ],
            [ Direction: Vertical,
              Done,
              [ 
                Contexts,
                Projects,
              ],
            ],
        ]
        "#;
        Layout::from_str(mock_layout, Arc::new(Mutex::new(ToDo::new(false)))).unwrap()
    }

    #[test]
    fn test_basic_movement() -> ToDoRes<()> {
        let mut l = mock_layout();
        let check_type = |widget_type, l: &Layout| -> ToDoRes<()> {
            let active = l.actual.as_ref().borrow().get_active_type();
            if active != widget_type {
                panic!("Active widget must be {:?} not {:?}.", widget_type, active)
            }
            Ok(())
        };

        check_type(WidgetType::List, &l)?;

        l.right();
        check_type(WidgetType::Done, &l)?;

        l.right();
        check_type(WidgetType::Done, &l)?;

        l.down();
        check_type(WidgetType::Context, &l)?;

        l.right();
        check_type(WidgetType::Project, &l)?;

        l.down();
        check_type(WidgetType::Project, &l)?;

        l.left();
        check_type(WidgetType::Context, &l)?;

        l.left();
        check_type(WidgetType::List, &l)?;

        l.right();
        check_type(WidgetType::Context, &l)?;

        l.left();
        check_type(WidgetType::List, &l)?;

        l.up();
        check_type(WidgetType::List, &l)?;

        Ok(())
    }

    #[test]
    fn test_from_string() -> ToDoRes<()> {
        let str_layout = r#"
            [
              Direction:Horizontal,
              Size: 50%,
              List: 50%,
              [ dIrEcTiOn: VeRtIcAl,
                Done,
                Hashtags: 50%,
              ],
              Projects: 50%,
            ]
            
            Direction: ERROR,
        "#;

        Layout::from_str(str_layout, Arc::new(Mutex::new(ToDo::new(false))))?;
        Ok(())
    }
}
