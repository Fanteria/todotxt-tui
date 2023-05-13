mod container;
mod widget;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use self::container::InitItem;
use self::container::Item;
use container::Container;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Rect},
    Frame,
};
use widget::{Widget, WidgetType};

pub struct Layout {
    root: Rc<RefCell<Container>>,
    active: WidgetType,
    actual: Rc<RefCell<Container>>,
}

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
        root.borrow_mut().update_chunks(chunk);
        let actual = Rc::clone(&root);

        Layout {
            root,
            active: WidgetType::List,
            actual,
        }
    }

    pub fn left(&self) {
       let actual = self.actual.as_ref().borrow() ;
        if actual.direction == Direction::Horizontal {
            let x = actual.actual_item();
        }
    

    }

    // pub fn left(&self) {
    //     let act = Layout::find_recursive(&self.layout, &self.active);
    //     let parent = match act {
    //         Some(i) => i.parent,
    //         None => return,
    //     };
    //     if parent.is_null() {
    //         return;
    //     } else {
    //         // let a = *parent;
    //     }
    //     // let parent = act.parent;
    // }

    // fn find_actual<'a>(&'a self) -> Option<&'a Widget> {
    //     Layout::find_recursive(&self.layout, &self.active)
    // }

    // fn find_recursive<'a>(
    //     container: &'a Rc<RefCell<Container>>,
    //     active: &WidgetType,
    // ) -> Option<&'a WidgetHolder> {
    //     for item in container.as_ref().borrow().items.iter() {
    //         match item {
    //             Item::Widget(widget) => {
    //                 if widget.widget.widget_type == *active {
    //                     return Some(&widget);
    //                 } else {
    //                     continue;
    //                 }
    //             }
    //             Item::Container(cont) => {
    //                 return Layout::find_recursive(&cont, active);
    //             }
    //         }
    //     }
    //     return None;
    // }

    pub fn update_chunks(&mut self, chunk: Rect) {
        self.root.borrow_mut().update_chunks(chunk);
    }

    pub fn render<B>(&self, f: &mut Frame<B>)
    where
        B: Backend,
    {
        Layout::render_layout_item(&self.root, &self.active, f)
    }

    fn render_layout_item<B>(layout: &Rc<RefCell<Container>>, active: &WidgetType, f: &mut Frame<B>)
    where
        B: Backend,
    {
        for item in layout.as_ref().borrow().items.iter() {
            match item {
                Item::Widget(holder) => holder.widget.draw(f, active),
                Item::Container(container) => {
                    Layout::render_layout_item(container.borrow(), active, f);
                }
            }
        }
    }
}
