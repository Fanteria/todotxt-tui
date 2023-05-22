pub mod container;
pub mod widget;

use self::container::InitItem;
use crate::error::ErrorToDo;
use container::Container;
use std::cell::RefCell;
use std::rc::Rc;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Direction::Horizontal, Direction::Vertical, Rect},
    Frame,
};
use widget::{Widget, WidgetType};

type RcCon = Rc<RefCell<Container>>;

pub struct Layout {
    root: Rc<RefCell<Container>>,
    actual: Rc<RefCell<Container>>,
}

impl Layout {
    pub fn new(chunk: Rect, actual: WidgetType) -> Layout {
        let input_widget = Widget::new(WidgetType::Input, "Input");
        let list_widget = Widget::new(WidgetType::List, "List");
        let done_widget = Widget::new(WidgetType::Done, "Done");
        let categories_widget = Widget::new(WidgetType::Project, "Projects");

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

        Layout { root, actual }
    }

    pub fn move_focus(
        container: RcCon,
        direction: &Direction,
        f: fn(RcCon) -> Option<RcCon>,
    ) -> Option<RcCon> {
        let move_to_parent = || {
            let mut c = container.borrow_mut();
            // c.parent.as_ref().and_then(|parent| {
            //     Layout::move_focus(parent.clone(), direction, f).map(|ret| {
            //         c.active = false;
            //         ret
            //     })
            // })

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

    pub fn left(&mut self) {
        let left = Layout::move_focus(
            Rc::clone(&self.actual),
            &Horizontal,
            Container::previous_item,
        );
        if let Some(actual) = left {
            self.actual = actual;
        }
    }

    pub fn right(&mut self) {
        let right = Layout::move_focus(Rc::clone(&self.actual), &Horizontal, Container::next_item);
        if let Some(actual) = right {
            self.actual = actual;
        }
    }

    pub fn up(&mut self) {
        let up = Layout::move_focus(Rc::clone(&self.actual), &Vertical, Container::previous_item);
        if let Some(actual) = up {
            self.actual = actual;
        }
    }

    pub fn down(&mut self) {
        let down = Layout::move_focus(Rc::clone(&self.actual), &Vertical, Container::next_item);
        if let Some(actual) = down {
            self.actual = actual;
        }
    }

    #[allow(dead_code)]
    pub fn select_widget(&mut self, widget_type: WidgetType) -> Result<(), ErrorToDo> {
        self.actual = Container::select_widget(self.root.clone(), widget_type)?;
        Ok(())
    }

    pub fn update_chunks(&mut self, chunk: Rect) {
        self.root.borrow_mut().update_chunks(chunk);
    }

    pub fn render<B>(&self, f: &mut Frame<B>)
    where
        B: Backend,
    {
        self.root.as_ref().borrow().render_recursive(f);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_movement() {}
}
