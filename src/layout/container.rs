use std::cell::RefCell;
use std::rc::Rc;

use tui::layout::{Constraint, Direction, Layout, Rect};
use super::widget::Widget;

pub enum Item {
    Container(Rc<RefCell<Container>>),
    Widget(WidgetHolder),
}

pub enum InitItem {
    Container(Rc<RefCell<Container>>),
    Widget(Widget),
}

pub struct WidgetHolder {
    pub widget: Widget,
    pub parent: Rc<RefCell<Container>>,
}

pub struct Container {
    pub items: Vec<Item>,
    pub layout: Layout,
    pub direction: Direction,
    pub parent: Option<Rc<RefCell<Container>>>,
    pub act_index: usize,
}

impl Container {
    pub fn new(
        items: Vec<InitItem>,
        constraints: Vec<Constraint>,
        direction: Direction,
        parent: Option<Rc<RefCell<Container>>>,
    ) -> Rc<RefCell<Container>> {
        let container = Rc::new(RefCell::new(Container {
            items: Vec::new(),
            layout: Layout::default()
                .direction(direction.clone())
                .constraints(constraints),
            direction,
            parent,
            act_index: 0,
        }));

        for item in items {
            match item {
                InitItem::Widget(widget) => {
                    container.as_ref().borrow_mut().items.push(Item::Widget(WidgetHolder {
                        widget,
                        parent: Rc::clone(&container),
                    }));
                }
                InitItem::Container(cont) => {
                    cont.borrow_mut().parent = Some(Rc::clone(&container));
                    container.borrow_mut().items.push(Item::Container(cont))
                }
            }
        }

        container
    }

    pub fn update_chunks(&mut self, chunk: Rect) {
        let chunks = self.layout.split(chunk);
        for (i, item) in self.items.iter_mut().enumerate() {
            match item {
                Item::Widget(holder) => holder.widget.update_chunk(chunks[i]),
                Item::Container(container) => {
                    container.borrow_mut().update_chunks(chunks[i])
                }
            }
        }
    }

    // pub fn next_item(&mut self) {
    //     self.act_index
    //
    // }
}
