use crate::todo::ToDo;

use super::UIEvent;
use crossterm::event::KeyEvent;

/// Trait for handling UI events.
pub trait HandleEvent {
    /// Get the UI event corresponding to a given key code.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code to map to a UI event.
    ///
    /// # Returns
    ///
    /// The UI event corresponding to the key code.
    fn get_event(&self, event: &KeyEvent) -> UIEvent;

    /// Handle a UI event.
    ///
    /// # Arguments
    ///
    /// * `event` - The UI event to handle.
    ///
    /// # Returns
    ///
    /// `true` if the event was successfully handled, `false` otherwise.
    fn handle_event(&mut self, event: UIEvent, todo: &mut ToDo) -> bool;

    /// Handle a key press event.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code representing the pressed key.
    ///
    /// # Returns
    ///
    /// `true` if the event was successfully handled, `false` otherwise.
    fn handle_key(&mut self, event: &KeyEvent, todo: &mut ToDo) -> bool {
        let event = self.get_event(event);
        log::trace!("EventHandler: Key '{:?}' cause event '{:?}'", event, event);
        self.handle_event(event, todo)
    }

    /// Handle a click event on a specified coordinates in the UI.
    ///
    /// # Arguments
    ///
    /// * `column` - `x` coordinate.
    /// * `row` - `y` coordinate.
    #[allow(unused_variables)]
    fn click(&mut self, column: usize, row: usize, todo: &ToDo) {}
}
