use super::{RCToDo, WidgetBase, WidgetType};
use crate::ui::{EventHandlerUI, HandleEvent, UIEvent};
use crate::CONFIG;
use crossterm::event::KeyCode;
use std::ops::{Deref, DerefMut};
use tui::widgets::ListState;

/// Represents a widget that displays a list of items.
pub struct WidgetList {
    base: WidgetBase,
    state: ListState,
    pub len: usize,
    first: usize,
    size: usize,
    shift: usize,
    event_handler: EventHandlerUI,
}

impl WidgetList {
    /// Creates a new `WidgetList` instance.
    ///
    /// # Parameters
    ///
    /// - `widget_type`: The type of widget.
    /// - `data`: A reference-counted mutex of `ToDo` data.
    ///
    /// # Returns
    ///
    /// A new `WidgetList` instance.
    pub fn new(widget_type: &WidgetType, data: RCToDo) -> Self {
        let mut def = Self {
            base: WidgetBase::new(widget_type, data),
            state: ListState::default(),
            len: 0,
            first: 0,
            size: 24,
            shift: 0,
            event_handler: CONFIG.list_keybind.clone(),
        };
        def.state.select(Some(0));
        def
    }

    /// Gets the currently selected item index.
    ///
    /// # Returns
    ///
    /// The index of the selected item.
    pub fn act(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    /// Gets the index of the item within the entire list, accounting for the first visible item.
    ///
    /// # Returns
    ///
    /// The adjusted index of the item.
    pub fn index(&self) -> usize {
        self.act() + self.first
    }

    /// Gets a clone of the list state.
    ///
    /// # Returns
    ///
    /// A clone of the list state.
    pub fn state(&self) -> ListState {
        self.state.clone()
    }

    /// Sets the shift value for the list.
    ///
    /// # Parameters
    ///
    /// - `shift`: The number of items to shift when navigating the list.
    pub fn set_shift(&mut self, shift: usize) {
        self.shift = shift;
    }

    /// Sets the size of the list widget.
    ///
    /// # Parameters
    ///
    /// - `size`: The size of the list widget.
    pub fn set_size(&mut self, size: u16) {
        self.size = size as usize;
    }

    /// Moves the selection down the list.
    pub fn down(&mut self) {
        let act = self.act();
        if self.len <= self.size {
            if self.len > act + 1 {
                self.state.select(Some(act + 1));
            }
        } else if self.size <= act + 1 + CONFIG.list_shift {
            if self.first + self.size < self.len {
                self.first += 1;
            } else if self.size > act + 1 {
                self.state.select(Some(act + 1));
            }
        } else {
            self.state.select(Some(act + 1));
        }
        log::trace!(
            "List go down: act: {}, size: {} len: {}, shift: {}",
            act,
            self.size,
            self.len,
            CONFIG.list_shift
        );
    }

    /// Moves the selection up the list.
    pub fn up(&mut self) {
        let act = self.act();
        if act <= CONFIG.list_shift {
            if self.first > 0 {
                self.first -= 1;
            } else if act > 0 {
                self.state.select(Some(act - 1));
            }
        } else {
            self.state.select(Some(act - 1));
        }
        log::trace!("List go up: act: {}", act);
    }

    /// Moves the selection to the next item and returns
    /// the indices of the old and new selections.
    ///
    /// # Returns
    ///
    /// An `Option` containing the indices of the (old, new) selections,
    /// or `None` if the list is at the end.
    pub fn next(&mut self) -> Option<(usize, usize)> {
        if (self.len <= self.size) && self.len <= self.act() + 1 {
            None
        } else {
            let old = self.index();
            self.down();
            Some((old, self.index()))
        }
    }

    /// Moves the selection to the previous item and returns the indices
    /// of the old and new selections.
    ///
    /// # Returns
    ///
    /// An `Option` containing the indices of the old and new selections,
    /// or `None` if the list is at the beginning.
    pub fn prev(&mut self) -> Option<(usize, usize)> {
        if self.act() == 0 {
            None
        } else {
            let old = self.index();
            self.up();
            Some((old, self.index()))
        }
    }

    /// Moves the selection to the first item in the list.
    pub fn first(&mut self) {
        self.state.select(Some(0));
        self.first = 0;
    }

    /// Moves the selection to the last item in the list.
    pub fn last(&mut self) {
        let shown_items = self.len - 1;
        if self.size > shown_items {
            self.first = 0;
            self.state.select(Some(shown_items));
        } else {
            self.first = shown_items - self.size;
            self.state.select(Some(self.size));
        }
    }

    /// Gets the range of items currently displayed in the list.
    ///
    /// # Returns
    ///
    /// A tuple containing the indices of the (first, last) items displayed.
    pub fn range(&self) -> (usize, usize) {
        (self.first, self.first + self.size)
    }
}

impl HandleEvent for WidgetList {
    fn get_event(&self, key: &KeyCode) -> UIEvent {
        self.event_handler.get_event(key)
    }

    fn handle_event(&mut self, event: UIEvent) -> bool {
        match event {
            UIEvent::ListDown => self.down(),
            UIEvent::ListUp => self.up(),
            UIEvent::ListFirst => self.first(),
            UIEvent::ListLast => self.last(),
            _ => return false,
        }
        true
    }
}

impl Deref for WidgetList {
    type Target = WidgetBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for WidgetList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
