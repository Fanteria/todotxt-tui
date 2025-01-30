use std::fmt::Debug;

use super::{super::Render, widget_base::WidgetBase, widget_type::WidgetType};
use crate::{
    todo::ToDo,
    ui::{HandleEvent, UIEvent},
};
use crossterm::event::KeyEvent;
use tui::{
    prelude::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

#[enum_dispatch]
pub trait State: Debug {
    /// Handles a UI event specific to the state and returns a boolean indicating
    /// whether the event was handled successfully.
    fn handle_event_state(&mut self, event: UIEvent, todo: &mut ToDo) -> bool;

    /// Renders the widget's state using the provided TUI frame.
    ///
    /// # Parameters
    ///
    /// - `f`: A mutable reference to the TUI frame used for rendering.
    fn render(&self, f: &mut Frame, todo: &ToDo);

    /// Retrieves a reference to the widget's base.
    fn get_base(&self) -> &WidgetBase;

    /// Retrieves a mutable reference to the widget's base.
    fn get_base_mut(&mut self) -> &mut WidgetBase;

    // Retrieves the block (border and title) for rendering the widget.
    fn get_block(&self) -> Block {
        let base = self.get_base();
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(base.title.clone())
            .border_type(base.border_type.into());
        if base.focus {
            block = block.border_style(Style::default().fg(base.active_color));
        }
        block
    }

    /// Called when the widget receives focus.
    fn focus_event(&mut self, _todo: &ToDo) -> bool {
        true
    }

    /// Called when the widget loses focus.
    fn unfocus_event(&mut self) {}

    /// Called when the widget's rendering area (chunk) is updated.
    fn update_chunk_event(&mut self) {}

    /// Handles the search by processing the given search string.
    ///
    /// # Arguments
    ///
    /// * `to_search` - A `String` representing the search input.
    ///
    #[allow(unused_variables)]
    fn search_event(&mut self, to_search: String) {}

    /// Clears the current search state, resetting any active search data.
    fn clear_search(&mut self) {}

    /// Retrieves an internal UI event based on a key code.
    /// This can be used for custom event handling within a state.
    ///
    /// # Parameters
    ///
    /// - `key`: The key code for which to generate an internal event.
    ///
    /// # Returns
    ///
    /// An internal UI event generated based on the provided key code.
    fn get_internal_event(&self, _: &KeyEvent) -> UIEvent {
        UIEvent::None
    }

    #[allow(unused_variables)]
    fn handle_click(&mut self, column: usize, row: usize, todo: &ToDo) {}

    /// Get the type of the widget.
    fn widget_type(&self) -> WidgetType;
}

impl<S: ?Sized + State> HandleEvent for S {
    fn get_event(&self, key_event: &KeyEvent) -> UIEvent {
        let event = self.get_internal_event(key_event);
        if event == UIEvent::None {
            self.get_base().event_handler.get_event(key_event)
        } else {
            event
        }
    }

    fn handle_event(&mut self, event: UIEvent, todo: &mut ToDo) -> bool {
        self.handle_event_state(event, todo)
    }

    fn click(&mut self, column: usize, row: usize, todo: &ToDo) {
        self.handle_click(column, row, todo)
    }
}

impl<S: ?Sized + State> Render for S {
    fn render(&self, f: &mut Frame, todo: &ToDo) {
        State::render(self, f, todo);
    }

    fn focus(&mut self, todo: &ToDo) -> bool {
        let ret = self.focus_event(todo);
        log::trace!(
            "Widget {} try to focus with result: {}",
            self.get_base().title,
            ret
        );
        self.get_base_mut().focus = ret;
        ret
    }

    fn unfocus(&mut self) {
        self.unfocus_event();
        log::trace!("Widget {} unfocus", self.get_base().title,);
        self.get_base_mut().focus = false;
    }

    fn update_chunk(&mut self, chunk: Rect) {
        self.get_base_mut().chunk = chunk;
        self.update_chunk_event();
    }
}
