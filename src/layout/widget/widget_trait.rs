use super::super::Render;
use super::widget_base::WidgetBase;
use crate::ui::{HandleEvent, UIEvent};
use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    prelude::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

#[enum_dispatch]
pub trait State {
    /// Handles a UI event specific to the state and returns a boolean indicating
    /// whether the event was handled successfully.
    ///
    /// # Parameters
    ///
    /// - `event`: The UI event to be handled.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the event was successfully handled.
    fn handle_event_state(&mut self, event: UIEvent) -> bool;

    /// Renders the widget's state using the provided TUI frame.
    ///
    /// # Parameters
    ///
    /// - `f`: A mutable reference to the TUI frame used for rendering.
    fn render<B: Backend>(&self, f: &mut Frame<B>);

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
    fn focus_event(&mut self) -> bool {
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
    fn get_internal_event(&self, _: &KeyCode) -> UIEvent {
        UIEvent::None
    }

    #[allow(unused_variables)]
    fn handle_click(&mut self, column: usize, row: usize) {}
}

impl<S: State> HandleEvent for S {
    fn get_event(&self, key: &KeyCode) -> UIEvent {
        let event = self.get_internal_event(key);
        if event == UIEvent::None {
            self.get_base().event_handler.get_event(key)
        } else {
            event
        }
    }

    fn handle_event(&mut self, event: UIEvent) -> bool {
        self.handle_event_state(event)
    }

    fn click(&mut self, column: usize, row: usize) {
        self.handle_click(column, row)
    }
}

impl<S: State> Render for S {
    fn render<B: Backend>(&self, f: &mut Frame<B>) {
        State::render(self, f);
    }

    fn focus(&mut self) -> bool {
        let ret = self.focus_event();
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
