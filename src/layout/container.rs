use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::error::{ErrorToDo, ErrorType};

use super::widget::{Widget, WidgetType};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

type RcCon = Rc<RefCell<Container>>;

pub enum Item {
    Container(RcCon),
    Widget(Holder),
}

pub enum InitItem {
    Container(RcCon),
    Widget(Widget),
}

pub struct Holder {
    pub widget: Widget,
    pub parent: RcCon,
}

pub struct Container {
    pub items: Vec<Item>,
    pub layout: Layout,
    pub direction: Direction,
    pub parent: Option<RcCon>,
    pub act_index: usize,
    pub active: bool,
}

#[allow(dead_code)]
impl Container {
    pub fn new(
        items: Vec<InitItem>,
        constraints: Vec<Constraint>,
        direction: Direction,
        parent: Option<RcCon>,
    ) -> RcCon {
        let container = Rc::new(RefCell::new(Container {
            items: Vec::new(),
            layout: Layout::default()
                .direction(direction.clone())
                .constraints(constraints),
            direction,
            parent,
            act_index: 0,
            active: false,
        }));

        for item in items {
            match item {
                InitItem::Widget(widget) => {
                    container
                        .as_ref()
                        .borrow_mut()
                        .items
                        .push(Item::Widget(Holder {
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
                Item::Container(container) => container.borrow_mut().update_chunks(chunks[i]),
            }
        }
    }

    pub fn actual_item(&self) -> &Item {
        &self.items[self.act_index]
    }

    pub fn update_actual(container: &RcCon) -> RcCon {
        let mut borrow = container.borrow_mut();
        match borrow.actual_item() {
            Item::Widget(_) => {
                borrow.active = true;
                return Rc::clone(container);
            }
            Item::Container(cont) => return Container::update_actual(cont),
        }
    }

    pub fn change_item(
        container: &RcCon,
        condition: fn(&Container) -> bool,
        change: fn(&mut Container),
    ) -> Option<RcCon> {
        if condition(&container.borrow()) {
            return None;
        }
        change(&mut container.borrow_mut());
        Some(Container::update_actual(container))
    }

    pub fn next_item(container: RcCon) -> Option<RcCon> {
        Container::change_item(
            &container,
            |c| c.act_index + 1 >= c.items.len(),
            |c| c.act_index += 1,
        )
    }

    pub fn previous_item(container: RcCon) -> Option<RcCon> {
        Container::change_item(&container, |c| c.act_index <= 0, |c| c.act_index -= 1)
    }

    pub fn select_widget(container: &RcCon, widget_type: &WidgetType) -> Result<RcCon, ErrorToDo> {
        let mut borrowed = container.borrow_mut();
        for (index, item) in borrowed.items.iter().enumerate() {
            match item {
                Item::Widget(holder) => {
                    if holder.widget.widget_type == *widget_type {
                        borrowed.active = true;
                        return Ok(Rc::clone(container));
                    }
                }
                Item::Container(container) => {
                    let cont = Container::select_widget(container, widget_type);
                    if cont.is_ok() {
                        borrowed.active = true;
                        borrowed.act_index = index;
                        return cont;
                    }
                }
            }
        }
        Err(ErrorToDo::new(
            ErrorType::WidgetDoesNotExist,
            "Selected widgent is not in layout",
        ))
    }

    pub fn render_recursive<B>(&self, f: &mut Frame<B>)
    where
        B: Backend,
    {
        for (index, item) in self.items.iter().enumerate() {
            match item {
                Item::Widget(holder) => holder
                    .widget
                    .draw(f, self.active && self.act_index == index),
                Item::Container(container) => container.borrow().render_recursive(f),
            }
        }
    }
}
