use tui::{backend::Backend, prelude::Rect, Frame};

/// A trait for rendering UI components.
///
/// The `Render` trait defines methods for rendering UI components within a TUI (Text-based User Interface).
pub trait Render {
    /// Render the UI component onto a TUI frame.
    ///
    /// This method is called to render the UI component onto a TUI frame.
    ///
    /// # Generic Parameters
    ///
    /// - `B`: The backend type used for rendering the UI component.
    ///
    /// # Parameters
    ///
    /// - `f`: A mutable reference to a TUI frame where the component should be rendered.
    fn render<B: Backend>(&self, f: &mut Frame<B>);

    /// Focus the UI component.
    ///
    /// This method is called to give focus to the UI component. Focusing a component
    /// typically means that it can now accept user input and respond to events.
    fn focus(&mut self);

    /// Unfocus the UI component.
    ///
    /// This method is called to remove focus from the UI component. An unfocused component
    /// typically does not respond to user input until it is focused again.
    fn unfocus(&mut self);

    /// Update the chunk (area) where the UI component should be rendered.
    ///
    /// This method is called to update the chunk (area) where the UI component should be rendered
    /// within the TUI frame. It allows the component to adapt to changes in size or position.
    ///
    /// # Parameters
    ///
    /// - `chunk`: The new rectangular area (chunk) where the component should be rendered.
    fn update_chunk(&mut self, chunk: Rect);
}
