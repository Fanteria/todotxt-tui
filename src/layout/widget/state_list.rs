use super::{widget_base::WidgetBase, widget_list::WidgetList, widget_trait::State};
use crate::{
    config::Config,
    todo::{search::Search, Parser, ToDo, ToDoData},
    ui::UIEvent,
    Result,
};
use crossterm::event::KeyEvent;
use tui::{style::Style, widgets::List, Frame};

/// Represents the state for a list widget that displays tasks.
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

    /// Gets the number of tasks in the list.
    ///
    /// # Returns
    ///
    /// The number of tasks in the list.
    pub fn len(&self) -> usize {
        self.base.data().len(self.data_type)
    }

    /// Swaps tasks in the list at the selected and previous indices.
    ///
    /// # Parameters
    ///
    /// - `first`: The index of the first task to swap.
    /// - `second`: The index of the second task to swap.
    fn swap_tasks(&mut self, first: usize, second: usize) {
        log::trace!("Swap tasks with indexes: {}, {}", first, second);
        self.base.data().swap_tasks(self.data_type, first, second);
    }

    /// Moves the currently selected task using the specified function.
    ///
    /// # Parameters
    ///
    /// - `move_fn`: The function to move the task (e.g., remove or move).
    fn move_task(&mut self, r#move: fn(&mut ToDo, ToDoData, usize)) {
        let index = self.base.index();
        log::info!("Remove task with index {index}.");
        r#move(&mut self.base.data(), self.data_type, index);
        let len = self.len();
        if len <= index && len > 0 {
            self.base.up();
        }
    }
}

impl State for StateList {
    fn handle_event_state(&mut self, event: UIEvent) -> bool {
        log::trace!("StateList handle event {event:?}");
        if self.base.handle_event(event, self.len()) {
            return true;
        }
        match event {
            UIEvent::SwapUpItem => {
                if let Some((first, second)) = self.base.prev() {
                    self.swap_tasks(first, second)
                }
            }
            UIEvent::SwapDownItem => {
                if let Some((first, second)) = self.base.next(self.len()) {
                    self.swap_tasks(first, second)
                }
            }
            UIEvent::RemoveItem => self.move_task(ToDo::remove_task),
            UIEvent::MoveItem => self.move_task(ToDo::move_task),
            UIEvent::Select => {
                log::trace!("Set item on index {} active.", self.base.index());
                self.base
                    .data()
                    .set_active(self.data_type, self.base.index());
            }
            UIEvent::NextSearch => {
                if let Some(to_search) = &self.base.to_search {
                    let next = {
                        let data = self.base.data();
                        let filtered = data.get_filtered_and_sorted(self.data_type);
                        let next = Search::find(
                            filtered.vec.iter().skip(self.base.index() + 1).enumerate(),
                            to_search,
                            |t| t.1 .1.subject.as_str(),
                        );
                        next.map(|next| next.0)
                    };
                    if let Some(next) = next {
                        log::debug!("Search next: {} times down", next);
                        for _ in 0..next + 1 {
                            self.base.down(self.len())
                        }
                    }
                }
            }
            UIEvent::PrevSearch => {
                if let Some(to_search) = &self.base.to_search {
                    let prev = {
                        let data = self.base.data();
                        let filtered = data.get_filtered_and_sorted(self.data_type);
                        let prev = Search::find(
                            filtered
                                .vec
                                .iter()
                                .rev()
                                .skip(filtered.vec.len() - self.base.index())
                                .enumerate(),
                            to_search,
                            |t| t.1 .1.subject.as_str(),
                        );
                        prev.map(|prev| prev.0)
                    };
                    if let Some(prev) = prev {
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

    fn render(&self, f: &mut Frame) {
        let data = self.base.data();
        let filtered = data.get_filtered_and_sorted(self.data_type);
        let (first, last) = self.base.range();
        let list = List::from(filtered.get_view(
            first..last,
            self.base.to_search.as_deref(),
            &self.parser,
            &data,
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

    fn focus_event(&mut self) -> bool {
        let len = self.len();
        // self.base.len = len;
        if self.base.act() >= len && len > 0 {
            self.base.last(self.len());
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

    fn handle_click(&mut self, column: usize, row: usize) {
        self.base.click(column, row, self.len());
    }
}
