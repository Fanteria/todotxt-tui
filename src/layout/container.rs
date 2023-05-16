use std::cell::RefCell;
use std::rc::Rc;

use crate::error::{ErrorToDo, ErrorType};

use super::widget::{Widget, WidgetType};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

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

#[allow(dead_code)]
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
                    container
                        .as_ref()
                        .borrow_mut()
                        .items
                        .push(Item::Widget(WidgetHolder {
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

    pub fn actual_widget(&self) -> Result<&Widget, ErrorToDo> {
        match self.actual_item() {
            Item::Widget(widget) => Ok(&widget.widget),
            Item::Container(_) => Err(ErrorToDo::new(
                ErrorType::ActualIsNotWidget,
                "Actual items is not widget.",
            )),
        }
    }

    pub fn next_item(&mut self) -> Option<&Item> {
        self.act_index += 1;
        if self.items.len() <= self.act_index {
            return None;
        }
        Some(&self.items[self.act_index])
    }

    pub fn previous_item(&mut self) -> Option<&Item> {
        if self.act_index <= 0 {
            return None;
        }
        self.act_index -= 1;
        Some(&self.items[self.act_index])
    }

    pub fn select_widget(
        container: &Rc<RefCell<Container>>,
        widget_type: &WidgetType,
    ) -> Result<Rc<RefCell<Container>>, ErrorToDo> {
        let mut borrowed = container.borrow_mut();
        // let index: usize;
        for (index, item) in borrowed.items.iter_mut().enumerate() {
            // borrowed.act_index = index;
            match item {
                Item::Widget(holder) => {
                    if holder.widget.widget_type == *widget_type {
                        // container.borrow_mut().act_index = index;
                        return Ok(Rc::clone(container));
                    }
                }
                Item::Container(container) => {
                    // container.borrow_mut().act_index = index;
                    let cont =  Container::select_widget(container, widget_type);
                    if cont.is_ok() {
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
                Item::Widget(holder) => holder.widget.draw(f, self.act_index == index),
                Item::Container(container) => container.borrow().render_recursive(f),
            }
        }
    }
}
