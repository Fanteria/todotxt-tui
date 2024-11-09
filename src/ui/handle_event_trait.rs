use super::UIEvent;
use crossterm::event::KeyCode;

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
    fn get_event(&self, key: &KeyCode) -> UIEvent;

    /// Handle a UI event.
    ///
    /// # Arguments
    ///
    /// * `event` - The UI event to handle.
    ///
    /// # Returns
    ///
    /// `true` if the event was successfully handled, `false` otherwise.
    fn handle_event(&mut self, event: UIEvent) -> bool;

    /// Handle a key press event.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code representing the pressed key.
    ///
    /// # Returns
    ///
    /// `true` if the event was successfully handled, `false` otherwise.
    fn handle_key(&mut self, key: &KeyCode) -> bool {
        let event = self.get_event(key);
        log::trace!("EventHandler: Key '{:?}' cause event '{:?}'", key, event);
        self.handle_event(event)
    }

    /// Handle a click event on a specified coordinates in the UI.
    ///
    /// # Arguments
    ///
    /// * `column` - `x` coordinate.
    /// * `row` - `y` coordinate.
    #[allow(unused_variables)]
    fn click(&mut self, column: usize, row: usize) {}
}
