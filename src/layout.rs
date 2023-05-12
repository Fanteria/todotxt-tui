mod container;
mod widget;

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
    layout: Container,
    active: WidgetType,
}

impl Layout {
    pub fn new(chunk: Rect) -> Layout {
        let input_widget = Widget::new(WidgetType::Input, "Input");
        let list_widget = Widget::new(WidgetType::List, "List");
        let done_widget = Widget::new(WidgetType::Done, "Done");

        let mut main_cont = Container::new(
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
        main_cont.update_chunks(chunk);

        Layout {
            layout: main_cont,
            active: WidgetType::List,
        }
    }

    // fn find_actual(&self) -> Option<Rc<RefCell<LayoutItem>>> {
    //     if let LayoutItem::Layout(layout) = self.layout.as_ref().borrow().deref() {
    //         return Layout::find_recursive(layout, &self.active);
    //     };
    //     None
    // }
    //
    // fn find_recursive(layout: &LayoutBox, actual: &WidgetType) -> Option<Rc<RefCell<LayoutItem>>> {
    //     for l in layout.chindrens.borrow().deref() {
    //         match l.as_ref().borrow().deref() {
    //             LayoutItem::Widget(widget) => {
    //                 if widget.widget_type == *actual {
    //                     return Some(Rc::clone(l));
    //                 }
    //             }
    //             LayoutItem::Layout(layout) => {
    //                 let result = Layout::find_recursive(layout, actual);
    //                 if result.is_some() {
    //                     return result;
    //                 }
    //             }
    //         }
    //     }
    //     None
    // }

    pub fn update_chunks(&mut self, chunk: Rect) {
        self.layout.update_chunks(chunk);
    }

    pub fn render<B>(&self, f: &mut Frame<B>)
    where
        B: Backend,
    {
        Layout::render_layout_item(&self.layout, &self.active, f)
    }

    fn render_layout_item<B>(layout: &Container, active: &WidgetType, f: &mut Frame<B>)
    where
        B: Backend,
    {
        for item in &layout.items {
            match item {
                Item::Widget(holder) => holder.widget.draw(f, active),
                Item::Container(container) => {
                    Layout::render_layout_item(container, active, f);
                }
            }
        }
    }
}
