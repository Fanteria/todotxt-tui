pub mod container;
pub mod widget;

use self::{
    container::{Container, InitItem, Item},
    widget::widget_type::WidgetType,
    widget::Widget,
};
use crate::{error::ErrorToDo, todo::ToDo};
use crossterm::event::KeyEvent;
use std::cell::RefCell;
use std::rc::Rc;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Direction::Horizontal, Direction::Vertical, Rect},
    Frame,
};

type RcCon = Rc<RefCell<Container>>;

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
            Item::Container(_) => false, // TODO return Errror
        }
    }

    pub fn select_widget(&mut self, widget_type: WidgetType) -> Result<(), ErrorToDo> {
        self.actual = Container::select_widget(self.root.clone(), widget_type)?;
        if let Item::Widget(w) = self.actual.borrow_mut().actual_item_mut() {
            println!("HELLO"); //TODO remove
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

    // fn check_active_type(container)

    #[test]
    fn test_basic_movement() -> Result<(), ErrorToDo> {
        let mut l = mock_layout();
        let check_type = |widget_type, l: &Layout| -> Result<(), ErrorToDo>{
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
}
