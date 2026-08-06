use tui::{
    buffer::Buffer,
    layout::Position,
    style::{Modifier, Style},
};

/// Text selection made with the mouse over the rendered frame.
#[derive(Debug, Clone, Copy)]
pub struct Selection {
    anchor: Position,
    head: Position,
}

impl Selection {
    /// Starts a new selection at the given position.
    pub fn new(position: Position) -> Self {
        Self {
            anchor: position,
            head: position,
        }
    }

    /// Moves the loose end of the selection to the given position.
    pub fn set_head(&mut self, position: Position) {
        self.head = position;
    }

    /// Returns true if the selection did not move from its start, which means
    /// a plain click and not a drag.
    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Returns the anchor and the head ordered in the reading order, so that
    /// dragging up or to the left works the same way as dragging down.
    fn bounds(&self) -> (Position, Position) {
        if (self.anchor.y, self.anchor.x) <= (self.head.y, self.head.x) {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }

    /// Highlights the selected cells in the buffer and returns their text.
    ///
    /// Every selected row is trimmed at the end, so the padding of the widgets
    /// does not end up in the clipboard.
    pub fn highlight(&self, buffer: &mut Buffer) -> String {
        let area = buffer.area;
        if area.is_empty() {
            return String::new();
        }
        let (start, end) = self.bounds();
        let (last_row, last_column) = (area.bottom() - 1, area.right() - 1);
        let mut text = String::new();
        for y in start.y.max(area.y)..=end.y.min(last_row) {
            let from = if y == start.y { start.x } else { area.x };
            let to = if y == end.y { end.x } else { last_column };
            let mut line = String::new();
            for x in from.max(area.x)..=to.min(last_column) {
                if let Some(cell) = buffer.cell_mut(Position::new(x, y)) {
                    cell.set_style(Style::default().add_modifier(Modifier::REVERSED));
                    line.push_str(cell.symbol());
                }
            }
            if y > start.y.max(area.y) {
                text.push('\n');
            }
            text.push_str(line.trim_end());
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn highlight_reads_in_reading_order() {
        let mut buffer = Buffer::with_lines(["abc ", "def "]);
        let mut selection = Selection::new(Position::new(1, 0));
        selection.set_head(Position::new(1, 1));

        assert_eq!(selection.highlight(&mut buffer), "bc\nde");

        let reversed = |x, y| {
            buffer
                .cell(Position::new(x, y))
                .unwrap()
                .modifier
                .contains(Modifier::REVERSED)
        };
        assert!(reversed(1, 0), "first selected cell is not highlighted");
        assert!(reversed(3, 0), "trailing space in range is not highlighted");
        assert!(reversed(1, 1), "last selected cell is not highlighted");
        assert!(!reversed(0, 0), "cell before the selection is highlighted");
        assert!(!reversed(2, 1), "cell after the selection is highlighted");
    }

    #[test]
    fn highlight_is_direction_agnostic() {
        let mut buffer = Buffer::with_lines(["abc ", "def "]);
        let mut selection = Selection::new(Position::new(1, 1));
        selection.set_head(Position::new(1, 0));

        assert_eq!(selection.highlight(&mut buffer), "bc\nde");
    }

    #[test]
    fn highlight_clamps_to_the_buffer() {
        let mut buffer = Buffer::with_lines(["abc"]);
        let mut selection = Selection::new(Position::new(1, 0));
        selection.set_head(Position::new(50, 10));

        assert_eq!(selection.highlight(&mut buffer), "bc");
    }
}
