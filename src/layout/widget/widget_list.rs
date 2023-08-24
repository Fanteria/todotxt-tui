use crossterm::event::{KeyCode, KeyEvent};
use tui::widgets::ListState;

const SHIFT: usize = 4;

pub struct WidgetList {
    state: ListState,
    first: usize,
    size: usize,
}

impl WidgetList {
    pub fn act(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    pub fn index(&self) -> usize {
        self.act() + self.first
    }

    pub fn state(&self) -> ListState {
        self.state.clone()
    }

    pub fn down(&mut self, len: usize) {
        let act = self.act();
        log::trace!("List go down: act: {}, len: {}", act, len);
        if len <= self.size {
            if len > act + 1 {
                self.state.select(Some(act + 1));
            }
        } else if self.size <= act + 1 + SHIFT {
            if self.first + self.size + 1 < len {
                self.first += 1;
            } else {
                if self.size > act + 1 {
                    self.state.select(Some(act + 1));
                }
            }
        } else {
            self.state.select(Some(act + 1));
        }
    }

    pub fn up(&mut self) {
        let act = self.act();
        log::trace!("List go up: act: {}", act);
        if act <= SHIFT {
            if self.first > 0 {
                self.first -= 1;
            } else {
                if act > 0 {
                    self.state.select(Some(act - 1));
                }
            }
        } else {
            self.state.select(Some(act - 1));
        }
    }

    /// (old, new)
    pub fn next(&mut self, len: usize) -> Option<(usize, usize)> {
        if (len <= self.size) && !(len > self.act() + 1) {
            None
        } else {
            let old = self.index();
            self.down(len);
            Some((old, self.index()))
        }
    }

    pub fn prev(&mut self) -> Option<(usize, usize)> {
        if self.act() <= 0 {
            None
        } else {
            let old = self.index();
            self.up();
            Some((old, self.index()))
        }
    }

    pub fn first(&mut self) {
        self.state.select(Some(0));
        self.first = 0;
    }

    pub fn last(&mut self) {
        let shown_items = self.size - 1;
        self.first = shown_items;
        self.state.select(Some(shown_items));
    }

    /// (first, last)
    pub fn range(&self) -> (usize, usize) {
        (self.first, self.first + self.size)
    }

    pub fn handle_key(&mut self, event: &KeyEvent, len: usize) -> bool {
        match event.code {
            KeyCode::Char('j') => self.down(len),
            KeyCode::Char('k') => self.up(),
            KeyCode::Char('g') => self.first(),
            KeyCode::Char('G') => self.last(),
            _ => return false,
        }
        true
    }
}

impl Default for WidgetList {
    fn default() -> Self {
        let mut def = Self {
            state: ListState::default(),
            first: 0,
            size: 24,
        };
        def.state.select(Some(0));
        def
    }
}
