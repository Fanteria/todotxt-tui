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

#[derive(PartialEq)]
pub enum WidgetType {
    Input,
    List,
    Done,
}

pub struct Widget {
    pub widget_type: WidgetType,
    pub chunk: Rect,
    pub parent: RefCell<Weak<RefCell<LayoutItem>>>,
    pub title: String,
}

impl Widget {
    pub fn new(
        widget_type: WidgetType,
        chunk: Rect,
        parent: RefCell<Weak<RefCell<LayoutItem>>>,
        title: &str,
    ) -> Widget {
        Widget {
            widget_type,
            chunk,
            parent,
            title: title.to_string(),
        }
    }

    pub fn draw<B>(&self, f: &mut Frame<B>, active: &WidgetType)
    where
        B: Backend,
    {
        let get_block = || {
            let mut block = Block::default()
                .borders(Borders::ALL)
                .title(self.title.clone())
                .border_type(BorderType::Rounded);
            if *active == self.widget_type {
                block = block.border_style(Style::default().fg(Color::Red));
            }
            block
        };

        match self.widget_type {
            WidgetType::Input => {
                f.render_widget(
                    Paragraph::new("Some text").block(get_block()),
                    self.chunk,
                );
            }
            WidgetType::List => {
                f.render_widget(get_block(), self.chunk);
            }
            WidgetType::Done => {
                f.render_widget(get_block(), self.chunk);
            }
        }
    }
}
