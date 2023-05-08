use crate::layout::LayoutItem;
use std::{cell::RefCell, rc::Weak};
use tui::{
    backend::Backend,
    layout::Rect,
    style::Color,
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum WidgetType {
    Input,
    List,
    Done,
    Categories,
}

pub struct Widget {
    pub widget_type: WidgetType,
    pub chunk: Rect,
    pub parent: RefCell<Weak<RefCell<LayoutItem>>>,
}

impl Widget {
    pub fn new(
        widget_type: WidgetType,
        chunk: Rect,
        parent: RefCell<Weak<RefCell<LayoutItem>>>,
    ) -> Widget {
        Widget {
            widget_type,
            chunk,
            parent,
        }
    }

    pub fn draw<B>(&self, f: &mut Frame<B>, active: &WidgetType)
    where
        B: Backend,
    {
        match self.widget_type {
            WidgetType::Input => {
                draw_input(f, &self.chunk, *active == self.widget_type);
            }
            WidgetType::List => {
                draw_list(f, &self.chunk, *active == self.widget_type);
            }
            WidgetType::Done => {
                draw_done(f, &self.chunk, *active == self.widget_type);
            }
            _ => {}
        }
    }
}

fn get_block(title: &str, active: bool) -> Block {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_type(BorderType::Rounded);
    if active {
        block = block.border_style(Style::default().fg(Color::Red));
    }
    block
}

fn draw_input<B>(f: &mut Frame<B>, chunk: &Rect, active: bool)
where
    B: Backend,
{
    f.render_widget(
        Paragraph::new("Some text").block(get_block("Input", active)),
        *chunk,
    );
}

fn draw_list<B>(f: &mut Frame<B>, chunk: &Rect, active: bool)
where
    B: Backend,
{
    f.render_widget(get_block("Todo list", active), *chunk);
}

fn draw_done<B>(f: &mut Frame<B>, chunk: &Rect, active: bool)
where
    B: Backend,
{
    f.render_widget(get_block("Done", active), *chunk);
}
