use crate::widget::{Widget, WidgetType};
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
    rc::Weak,
    vec::Vec, borrow::Borrow,
};
use tui::{
    backend::Backend,
    layout,
    layout::{Constraint, Direction, Layout as tuiLayout, Rect},
    Frame,
};

pub enum LayoutItem {
    Layout(LayoutBox),
    Widget(Widget),
}

pub struct LayoutBox {
    pub chindrens: RefCell<Vec<Rc<RefCell<LayoutItem>>>>,
    pub layout: layout::Layout,
    pub direction: Direction,
    pub parent: RefCell<Weak<RefCell<LayoutItem>>>,
}

impl LayoutBox {
    pub fn new(
        chindrens: RefCell<Vec<Rc<RefCell<LayoutItem>>>>,
        layout: layout::Layout,
        direction: Direction,
        parent: RefCell<Weak<RefCell<LayoutItem>>>,
    ) -> LayoutBox {
        LayoutBox {
            chindrens,
            layout,
            direction,
            parent,
        }
    }
}

pub struct Layout {
    layout: Rc<RefCell<LayoutItem>>,
    active: WidgetType,
}

impl Layout {
    pub fn new(chunk: Rect) -> Layout {
        let main_layout = tuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(30)].as_ref());
        let main_chunks = main_layout.split(chunk);

        let body_layout = tuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref());
        let body_chunks = body_layout.split(main_chunks[1]);

        let make_widget = |chunk, widget_type, title| {
            Rc::new(RefCell::new(LayoutItem::Widget(Widget::new(
                widget_type,
                chunk,
                RefCell::new(Weak::new()),
                title,
            ))))
        };
        let input_widget = make_widget(main_chunks[0], WidgetType::Input, "Input");
        let list_widget = make_widget(body_chunks[0], WidgetType::List, "List");
        let done_widget = make_widget(body_chunks[1], WidgetType::Done, "Done");

        let make_layout = |vec, layout, direction| {
            Rc::new(RefCell::new(LayoutItem::Layout(LayoutBox::new(
                RefCell::new(vec),
                layout,
                direction,
                RefCell::new(Weak::new()),
            ))))
        };
        let body_layout_box = make_layout(
            vec![Rc::clone(&list_widget), Rc::clone(&done_widget)],
            body_layout,
            Direction::Horizontal,
        );

        let register_child = |widget: Rc<RefCell<LayoutItem>>,
                              data_box: &Rc<RefCell<LayoutItem>>| {
            match widget.as_ref().borrow().deref() {
                LayoutItem::Layout(layout_box) => {
                    *layout_box.parent.borrow_mut() = Rc::downgrade(&data_box);
                }
                LayoutItem::Widget(widget) => {
                    *widget.parent.borrow_mut() = Rc::downgrade(&data_box);
                }
            }
        };
        register_child(list_widget, &body_layout_box);
        register_child(done_widget, &body_layout_box);

        let main_layout_box = make_layout(
            vec![Rc::clone(&input_widget), Rc::clone(&body_layout_box)],
            main_layout,
            Direction::Vertical,
        );
        register_child(input_widget, &main_layout_box);
        register_child(body_layout_box, &main_layout_box);

        Layout {
            layout: main_layout_box,
            active: WidgetType::List,
        }
    }

    pub fn left(&self) -> Option<Rc<RefCell<LayoutItem>>> {
        let item = match self.find_actual() {
            Some(s) => s,
            None => return None,
        };
        let reference = item.as_ref().borrow();
        let parent = match reference.deref() {
            LayoutItem::Widget(widget) => &widget.parent,
            LayoutItem::Layout(_) => return None,
        };
        let reference = parent.borrow();
        let reference = reference.upgrade().unwrap();
        let reference = reference.as_ref().borrow();
        let direction = match reference.deref() {
            LayoutItem::Widget(_) => return None,
            LayoutItem::Layout(layout) => &layout.direction,
        };
        match direction {
            Direction::Vertical => {},
            Direction::Horizontal => {},
        }

        None
    }

    fn find_actual(&self) -> Option<Rc<RefCell<LayoutItem>>> {
        if let LayoutItem::Layout(layout) = self.layout.as_ref().borrow().deref() {
            return Layout::find_recursive(layout, &self.active);
        };
        None
    }

    fn find_recursive(layout: &LayoutBox, actual: &WidgetType) -> Option<Rc<RefCell<LayoutItem>>> {
        for l in layout.chindrens.borrow().deref() {
            match l.as_ref().borrow().deref() {
                LayoutItem::Widget(widget) => {
                    if widget.widget_type == *actual {
                        return Some(Rc::clone(l));
                    }
                }
                LayoutItem::Layout(layout) => {
                    let result = Layout::find_recursive(layout, actual);
                    if result.is_some() {
                        return result;
                    }
                }
            }
        }
        None
    }

    pub fn update_chunk(&mut self, chunk: Rect) {
        Layout::update_chunks(&mut self.layout.as_ref().borrow_mut().deref_mut(), chunk);
    }

    fn update_chunks(layout: &mut LayoutItem, chunk: Rect) {
        match layout {
            LayoutItem::Widget(widget) => widget.chunk = chunk,
            LayoutItem::Layout(layout_box) => {
                let chunks = layout_box.layout.split(chunk);
                for (chunk, item) in chunks
                    .iter()
                    .zip(layout_box.chindrens.borrow_mut().iter_mut())
                {
                    Layout::update_chunks(item.as_ref().borrow_mut().deref_mut(), *chunk);
                }
            }
        }
    }

    pub fn render<B>(&self, f: &mut Frame<B>)
    where
        B: Backend,
    {
        Layout::render_layout_item(&self.layout.as_ref().borrow().deref(), &self.active, f)
    }

    fn render_layout_item<B>(layout: &LayoutItem, active: &WidgetType, f: &mut Frame<B>)
    where
        B: Backend,
    {
        match layout {
            LayoutItem::Widget(widget) => widget.draw(f, active),
            LayoutItem::Layout(layout_box) => {
                for l in layout_box.chindrens.borrow().deref() {
                    Layout::render_layout_item(l.as_ref().borrow().deref(), active, f);
                }
            }
        }
    }
}
