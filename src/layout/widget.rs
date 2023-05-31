use super::widget_state::State;
use super::widget_state::WidgetState;
use super::widget_type::WidgetType;
use crate::todo::ToDo;
use crossterm::event::KeyEvent;
use std::rc::Rc;
use tui::{backend::Backend, layout::Rect, Frame};

pub struct Widget {
    pub widget_type: WidgetType,
    pub chunk: Rect,
    pub title: String,
    pub data: Rc<ToDo>,
    state: WidgetState,
}

impl Widget {
    pub fn new(widget_type: WidgetType, title: &str, data: Rc<ToDo>) -> Widget {
        Widget {
            widget_type,
            chunk: Rect {
                width: 0,
                height: 0,
                x: 0,
                y: 0,
            },
            title: title.to_string(),
            data: data.clone(),
            state: WidgetState::new(&widget_type),
        }
    }

    pub fn update_chunk(&mut self, chunk: Rect) {
        self.chunk = chunk;
    }

    pub fn handle_key(&mut self, event: &KeyEvent) {
        self.state.handle_key(event);
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, active: bool) {
        self.state
            .render(f, active, &self.title, self.data.as_ref(), self.chunk);
        // let get_block = || {
        //     let mut block = Block::default()
        //         .borders(Borders::ALL)
        //         .title(self.title.clone())
        //         .border_type(BorderType::Rounded);
        //     if active {
        //         block = block.border_style(Style::default().fg(CONFIG.active_color));
        //     }
        //     block
        // };
        // let mut list_state = ListState::default();
        // list_state.select(Some(0));
        // list_state.select(list_state.selected().and_then(|i| Some(i + 1)));
        //
        // match self.widget_type {
        //     WidgetType::Input => {
        //         f.render_widget(Paragraph::new("Some text").block(get_block()), self.chunk);
        //     }
        //     WidgetType::List => {
        //         let list = List::new(self.data.pending.clone())
        //             .block(get_block())
        //             .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        //             .highlight_symbol(">>");
        //         f.render_stateful_widget(list, self.chunk, &mut list_state);
        //     }
        //     WidgetType::Done => {
        //         let list = List::new(self.data.done.clone())
        //             .block(get_block())
        //             .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        //             .highlight_symbol(">>");
        //         f.render_widget(list, self.chunk);
        //     }
        //     WidgetType::Project => {
        //         let list = List::new(self.data.get_projects())
        //             .block(get_block())
        //             .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        //             .highlight_symbol(">>");
        //         f.render_widget(list, self.chunk);
        //     }
        //     WidgetType::Context => {
        //         f.render_widget(get_block(), self.chunk);
        //     }
        // }
    }
}
