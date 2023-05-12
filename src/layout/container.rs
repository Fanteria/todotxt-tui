use tui::layout::{Constraint, Direction, Layout, Rect};
use super::widget::Widget;

pub enum Item {
    Container(Container),
    Widget(WidgetHolder),
}

pub enum InitItem {
    Container(Container),
    Widget(Widget),
}

pub struct WidgetHolder {
    pub widget: Widget,
    pub parent: *const Container,
}

pub struct Container {
    pub items: Vec<Item>,
    pub layout: Layout,
    pub direction: Direction,
    pub parent: Option<*const Container>,
}

impl Container {
    pub fn new(
        items: Vec<InitItem>,
        constraints: Vec<Constraint>,
        direction: Direction,
        parent: Option<*const Container>,
    ) -> Container {
        let mut container = Container {
            items: Vec::new(),
            layout: Layout::default()
                .direction(direction.clone())
                .constraints(constraints),
            direction,
            parent,
        };

        let mut items_vec = Vec::new();
        for item in items {
            match item {
                InitItem::Widget(widget) => {
                    items_vec.push(Item::Widget(WidgetHolder {
                        widget,
                        parent: &container,
                    }));
                }
                InitItem::Container(mut cont) => {
                    cont.parent = Some(&container);
                    items_vec.push(Item::Container(cont))
                }
            }
        }

        container.items = items_vec;
        container
    }

    pub fn update_chunks(&mut self, chunk: Rect) {
        let chunks = self.layout.split(chunk);
        for (i, item) in self.items.iter_mut().enumerate() {
            match item {
                Item::Widget(holder) => holder.widget.update_chunk(chunks[i]),
                Item::Container(container) => {
                    container.update_chunks(chunks[i])
                }
            }
        }
    }
}
