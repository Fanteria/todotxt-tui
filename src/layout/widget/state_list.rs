use super::{widget_base::WidgetBase, widget_list::WidgetList, widget_trait::State};
use crate::{
    config::Config,
    todo::{search::Searchable, Parser, ToDo, ToDoData},
    ui::UIEvent,
    Result,
};
use crossterm::event::KeyEvent;
use tui::{style::Style, widgets::List, Frame};

/// Represents the state for a list widget that displays tasks.
#[derive(Debug)]
pub struct StateList {
    base: WidgetList,
    style: Style,
    parser: Parser,
    pub data_type: ToDoData,
}

impl StateList {
    /// Creates a new `StateList` instance.
    ///
    /// # Parameters
    ///
    /// - `base`: The base properties shared among different widget types.
    /// - `data_type`: The type of task data to display (e.g., Pending or Done tasks).
    /// - `style`: The style used to render the list widget.
    ///
    /// # Returns
    ///
    /// A new `StateList` instance.
    pub fn new(base: WidgetList, data_type: ToDoData, config: &Config) -> Result<Self> {
        Ok(Self {
            base,
            style: config
                .active_color_config
                .get_active_style(&data_type)
                .get_style(),
            parser: Parser::new(
                match data_type {
                    ToDoData::Pending => &config.list_config.pending_format,
                    ToDoData::Done => &config.list_config.done_format,
                },
                config.styles.clone(),
            )?,
            data_type,
        })
    }

    /// Moves the currently selected task using the specified function.
    ///
    /// # Parameters
    ///
    /// - `move_fn`: The function to move the task (e.g., remove or move).
    fn move_task(&mut self, r#move: fn(&mut ToDo, ToDoData, usize), todo: &mut ToDo) {
        let index = self.base.index();
        log::info!("Remove task with index {index}.");
        r#move(todo, self.data_type, index);
        let len = todo.len(self.data_type);
        if len <= index && len > 0 {
            self.base.up();
        }
    }
}

impl State for StateList {
    fn handle_event_state(&mut self, event: UIEvent, todo: &mut ToDo) -> bool {
        log::trace!("StateList handle event {event:?}");
        if self.base.handle_event(event, todo.len(self.data_type)) {
            todo.set_actual(self.data_type, self.base.index());
            return true;
        }
        match event {
            UIEvent::SwapUpItem => {
                if let Some((first, second)) = self.base.prev() {
                    todo.swap_tasks(self.data_type, first, second)
                }
            }
            UIEvent::SwapDownItem => {
                if let Some((first, second)) = self.base.next(todo.len(self.data_type)) {
                    todo.swap_tasks(self.data_type, first, second)
                }
            }
            UIEvent::RemoveItem => self.move_task(ToDo::remove_task, todo),
            UIEvent::MoveItem => self.move_task(ToDo::move_task, todo),
            UIEvent::Select => {
                log::trace!("Set item on index {} active.", self.base.index());
                todo.set_active(self.data_type, self.base.index());
            }
            UIEvent::NextSearch => {
                if let Some(to_search) = &self.base.to_search {
                    if let Some(next) = todo
                        .get_filtered_and_sorted(self.data_type)
                        .next_search_index(to_search, self.base.index() + 1)
                    {
                        log::debug!("Search next: {} times down", next);
                        for _ in 0..next + 1 {
                            self.base.down(todo.len(self.data_type))
                        }
                    }
                }
            }
            UIEvent::PrevSearch => {
                if let Some(to_search) = &self.base.to_search {
                    if let Some(prev) = todo
                        .get_filtered_and_sorted(self.data_type)
                        .prev_search_index(to_search, self.base.index())
                    {
                        log::debug!("Search prev: {} times up", prev);
                        for _ in 0..prev + 1 {
                            self.base.up()
                        }
                    }
                }
            }
            _ => return false,
        }
        true
    }

    fn render(&self, f: &mut Frame, todo: &ToDo) {
        let filtered = todo.get_filtered_and_sorted(self.data_type);
        let (first, last) = self.base.range();
        let list = List::from(filtered.get_view(
            first..last,
            self.base.to_search.as_deref(),
            &self.parser,
            todo,
        ))
        .block(self.get_block());
        if !self.base.focus {
            f.render_widget(list, self.base.chunk)
        } else {
            let list = list.highlight_style(self.style);
            f.render_stateful_widget(list, self.base.chunk, &mut self.base.state());
        }
    }

    fn get_base(&self) -> &WidgetBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut WidgetBase {
        &mut self.base
    }

    fn focus_event(&mut self, todo: &ToDo) -> bool {
        if self.base.act() >= todo.len(self.data_type) && todo.len(self.data_type) > 0 {
            self.base.last(todo.len(self.data_type));
        }
        true
    }

    fn search_event(&mut self, to_search: String) {
        self.base.set_search(to_search);
    }

    fn clear_search(&mut self) {
        self.base.clear_search();
    }

    fn update_chunk_event(&mut self) {
        self.base.set_size(self.base.chunk.height - 2); // Two chars are borders.
    }

    fn get_internal_event(&self, event: &KeyEvent) -> UIEvent {
        self.base.get_event(event)
    }

    fn handle_click(&mut self, column: usize, row: usize, todo: &ToDo) {
        self.base.click(column, row, todo.len(self.data_type));
    }

    fn widget_type(&self) -> super::widget_type::WidgetType {
        self.data_type.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{widget::widget_type::WidgetType, Render};
    use test_log::test;
    use tui::{backend::TestBackend, prelude::Rect, Terminal};

    fn make_list(data_type: ToDoData) -> (StateList, Config) {
        let config = Config::default();
        let base = WidgetList::new(
            &match data_type {
                ToDoData::Pending => WidgetType::List,
                ToDoData::Done => WidgetType::Done,
            },
            &config,
        );
        let list = StateList::new(base, data_type, &config).unwrap();
        (list, config)
    }

    #[test]
    fn render_empty_pending_list() -> Result<()> {
        let (mut list, _) = make_list(ToDoData::Pending);
        let area = Rect::new(0, 0, 20, 5);
        list.update_chunk(area);

        let todo = ToDo::default();

        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| State::render(&list, f, &todo))?;

        terminal.backend().assert_buffer_lines([
            "╭list──────────────╮",
            "│                  │",
            "│                  │",
            "│                  │",
            "╰──────────────────╯",
        ]);
        Ok(())
    }

    #[test]
    fn render_pending_tasks() -> Result<()> {
        let (mut list, _) = make_list(ToDoData::Pending);
        let area = Rect::new(0, 0, 20, 5);
        list.update_chunk(area);

        let mut todo = ToDo::default();
        todo.new_task("Alpha")?;
        todo.new_task("Beta")?;

        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| State::render(&list, f, &todo))?;

        terminal.backend().assert_buffer_lines([
            "╭list──────────────╮",
            "│Alpha             │",
            "│Beta              │",
            "│                  │",
            "╰──────────────────╯",
        ]);
        Ok(())
    }

    #[test]
    fn render_done_tasks() -> Result<()> {
        let (mut list, _) = make_list(ToDoData::Done);
        let area = Rect::new(0, 0, 20, 5);
        list.update_chunk(area);

        let mut todo = ToDo::default();
        todo.new_task("x Finished")?;

        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| State::render(&list, f, &todo))?;

        terminal.backend().assert_buffer_lines([
            "╭done──────────────╮",
            "│Finished          │",
            "│                  │",
            "│                  │",
            "╰──────────────────╯",
        ]);
        Ok(())
    }

    #[test]
    fn render_focused_highlights_selected() -> Result<()> {
        let (mut list, _) = make_list(ToDoData::Pending);
        let area = Rect::new(0, 0, 20, 5);
        list.update_chunk(area);

        let mut todo = ToDo::default();
        todo.new_task("Alpha")?;
        todo.new_task("Beta")?;
        list.focus(&ToDo::default());

        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| State::render(&list, f, &todo))?;

        // When focused, the first item should have a highlight style applied.
        // Check that the content is still there; the highlight is a style
        // attribute on the buffer cells.
        let buf = terminal.backend().buffer();
        // Row 1 (first item inside border) should have the active color style
        let cell = buf.cell((1, 1)).unwrap();
        assert_eq!(cell.symbol(), "A");
        assert_eq!(cell.bg, tui::style::Color::LightRed);
        assert_eq!(cell.fg, tui::style::Color::Reset);
        Ok(())
    }
}
