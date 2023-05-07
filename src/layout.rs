use crate::widget::{Widget, WidgetType};
use std::vec::Vec;
use tui::{
    backend::Backend,
    layout,
    layout::{Constraint, Direction, Layout as tuiLayout, Rect},
    Frame,
};

#[allow(dead_code)]
enum LayoutItem {
    Layout(Vec<LayoutItem>, layout::Layout),
    Widget(Widget),
}

#[allow(dead_code)]
pub struct Layout {
    layout: LayoutItem,
}

#[allow(dead_code)]
impl Layout {
    pub fn new(chunk: Rect) -> Layout {
        let main_layout = tuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(30)].as_ref());
        let main_chunks = main_layout.split(chunk);
        let input_widget = Widget::new(WidgetType::Input, main_chunks[0]);

        let body_layout = tuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref());
        let body_chunks = body_layout.split(main_chunks[1]);
        let list_widget = Widget::new(WidgetType::List, body_chunks[0]);
        let done_widget = Widget::new(WidgetType::Done, body_chunks[1]);

        let mut body_vector: Vec<LayoutItem> = Vec::new();
        body_vector.push(LayoutItem::Widget(list_widget));
        body_vector.push(LayoutItem::Widget(done_widget));

        let mut main_vector: Vec<LayoutItem> = Vec::new();
        main_vector.push(LayoutItem::Widget(input_widget));
        main_vector.push(LayoutItem::Layout(body_vector, body_layout));

        Layout {
            layout: LayoutItem::Layout(main_vector, main_layout),
        }
    }

    pub fn update_chunk(&mut self, chunk: Rect) {
        update_chunks(&mut self.layout, chunk);
    }

    pub fn render<B>(&self, f: &mut Frame<B>)
    where
        B: Backend,
    {
        render_layout_item(&self.layout, f);
    }
}

fn update_chunks(layout: &mut LayoutItem, chunk: Rect) {
    match layout {
        LayoutItem::Widget(widget) => {widget.chunk = chunk}
        LayoutItem::Layout(vec, layout) => {
            let chunks = layout.split(chunk);
            for (chunk, item) in chunks.iter().zip(vec.iter_mut()) {
                update_chunks(item, *chunk);
            }
        },
    }

}

fn render_layout_item<B>(layout: &LayoutItem, f: &mut Frame<B>)
where
    B: Backend,
{
    match layout {
        LayoutItem::Widget(widget) => widget.draw(f),
        LayoutItem::Layout(vec, _layout) => for l in vec {
            render_layout_item(l, f);
        },
    }
}
