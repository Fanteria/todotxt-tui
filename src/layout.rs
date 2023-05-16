mod container;
mod widget;

use std::cell::RefCell;
use std::rc::Rc;

use crate::error::{ErrorToDo, ErrorType};

use self::container::{InitItem, Item};
use container::Container;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Rect},
    Frame,
};
use widget::{Widget, WidgetType};

#[allow(dead_code)]
pub struct Layout {
    root: Rc<RefCell<Container>>,
    active: WidgetType,
    actual: Rc<RefCell<Container>>,
}

#[allow(dead_code)]
impl Layout {
    pub fn new(chunk: Rect) -> Layout {
        let input_widget = Widget::new(WidgetType::Input, "Input");
        let list_widget = Widget::new(WidgetType::List, "List");
        let done_widget = Widget::new(WidgetType::Done, "Done");

        let root = Container::new(
            vec![
                InitItem::Widget(input_widget),
                InitItem::Container(Container::new(
                    vec![InitItem::Widget(list_widget), InitItem::Widget(done_widget)],
                    vec![Constraint::Percentage(50), Constraint::Percentage(50)],
                    Direction::Horizontal,
                    None,
                )),
            ],
            vec![Constraint::Length(3), Constraint::Percentage(30)],
            Direction::Vertical,
            None,
        );
        let actual = Container::select_widget(&root, &WidgetType::List).unwrap(); // TODO
        root.borrow_mut().update_chunks(chunk);

        Layout {
            root,
            active: WidgetType::List,
            actual,
        }
    }

    pub fn left(&self) {
        let actual = self.actual.as_ref().borrow();
        if actual.direction == Direction::Horizontal {
            // let x = actual.actual_item();
        }
    }

    pub fn right(&mut self) -> Result<(), ErrorToDo> {
        let mut actual = self.actual.as_ref().borrow_mut();
        let item = actual.next_item();
        // println!("move right: {:?}", actual.actual_widget()?.widget_type);
        // actual.actual_widget()?.draw(f, false);
        // if actual.direction == Direction::Horizontal {
        // println!("Horizontal");
        //     actual.actual_widget()?.draw(f, false);
        //     let next_item = actual.next_item();
        //     match next_item {
        //         Some(item) => match item {
        //             Item::Widget(holder) => holder.widget.draw(f, true),
        //             Item::Container(_) => {} // TODO
        //         },
        //         None => {}
        //     }
        // }
        Ok(())
    }

    pub fn select_widget(&mut self, widget_type: &WidgetType) -> Result<(), ErrorToDo> {
        self.actual = Container::select_widget(&self.root, widget_type)?;
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
