mod container;
mod render_trait;
pub mod widget;

use crate::{
    config::Config, layout::widget::State, todo::ToDo, ui::HandleEvent, Result, ToDoError,
};
use container::Container;
use core::panic;
use crossterm::event::KeyEvent;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use std::{fmt::Debug, str::FromStr, sync::Arc, sync::Mutex};
use tui::{
    layout::{Constraint, Direction, Rect},
    Frame,
};
use widget::{widget_type::WidgetType, Widget};

#[derive(Parser)]
#[grammar = "./layout/grammar.pest"]
struct LayoutParser;

pub use render_trait::Render;

struct Holder {
    container: usize,    // container
    widgets: Vec<usize>, // widget
}
impl Holder {
    fn new(l: &Layout) -> Holder {
        Holder {
            container: l.act,
            widgets: l.containers.iter().map(|c| c.get_index()).collect(),
        }
    }
    fn unfocus(&self, l: &mut Layout) {
        match l.containers[self.container].get_widget_mut(self.widgets[self.container]) {
            Some(widget) if widget.get_base().focus => widget.unfocus(),
            _ => {}
        }
    }
    fn set_old_back(&self, l: &mut Layout) {
        l.act = self.container;
        l.containers
            .iter_mut()
            .zip(self.widgets.iter())
            .for_each(|(c, i)| {
                c.set_index(*i);
            });
    }
}

/// Represents the layout of the user interface.
///
/// The `Layout` struct defines the layout of the user interface for the todo-tui application. It
/// consists of a tree of containers and widgets, which are used to organize and display the various
/// components of the application.
#[derive(Debug)]
pub struct Layout {
    containers: Vec<Container>,
    act: usize,
}

impl Layout {
    /// Create a new `Layout` from a template string.
    ///
    /// This function parses a template string and creates a new `Layout` instance based on the
    /// specified template. The template string defines the layout of the user interface, including
    /// the arrangement of containers and widgets.
    pub fn from_str(template: &str, data: Arc<Mutex<ToDo>>, config: &Config) -> Result<Self> {
        fn read_block(
            conts: &mut Vec<Container>,
            parent: Option<usize>,
            block: Pair<'_, Rule>,
            data: Arc<Mutex<ToDo>>,
            config: &Config,
        ) -> Result<usize> {
            let index = conts.len();
            conts.push(Container::default());
            conts[index].parent = parent;
            // Set direction toggling.
            let direction = match conts[index].parent.map(|i| conts[i].get_direction()) {
                Some(Direction::Vertical) => Direction::Horizontal,
                _ => Direction::Vertical,
            };
            conts[index].set_direction(direction);
            let mut constrains = vec![];
            let mut act_constrain = None;
            for inner in block.into_inner() {
                use Constraint::*;
                use Direction::*;
                let get_num = |i: &Pair<'_, Rule>| -> u16 { i.as_str().parse().unwrap() };
                match inner.as_rule() {
                    Rule::directory_horizontal => conts[index].set_direction(Horizontal),
                    Rule::directory_vertical => conts[index].set_direction(Vertical),
                    Rule::size_raw => act_constrain = Some(Length(get_num(&inner))),
                    Rule::size_percentage => act_constrain = Some(Percentage(get_num(&inner))),
                    Rule::widget => {
                        // First item is mandatory name and second item is optional size.
                        let widget_blocks = inner.into_inner().collect::<Vec<_>>();
                        constrains.push(match widget_blocks.get(1) {
                            Some(size) if size.as_rule() == Rule::size_raw => {
                                Constraint::Length(get_num(size))
                            }
                            Some(size) if size.as_rule() == Rule::size_percentage => {
                                Constraint::Percentage(get_num(size))
                            }
                            _ => Constraint::Percentage(50),
                        });
                        conts[index].add_widget(Widget::new(
                            WidgetType::from_str(widget_blocks[0].as_str())?,
                            data.clone(),
                            config,
                        )?);
                    }
                    Rule::block => {
                        constrains.push(act_constrain.take().unwrap_or(Constraint::Percentage(50)));
                        let next_item =
                            read_block(conts, Some(index), inner, data.clone(), config)?;
                        conts[index].add_cont(next_item);
                    }
                    _ => unreachable!(),
                }
            }
            conts[index].set_constraints(constrains);
            Ok(index)
        }
        let parsed = LayoutParser::parse(Rule::layout, template)
            .map_err(|e| ToDoError::FailedToParseLayout(Box::new(e)))?
            .next()
            .unwrap(); // It is parsed by pest, its is safe to get first item

        // Here are two inners in, first is outer block and second is EOI.
        let outer_block = parsed.into_inner().collect::<Vec<_>>().remove(0);
        let mut containers: Vec<Container> = vec![];
        read_block(&mut containers, None, outer_block, data, config)?;

        let mut layout = Self { containers, act: 0 };
        Container::actualize_layout(&mut layout);
        layout.act_mut().actual_mut().unwrap().focus();
        Ok(layout)
    }

    fn act(&self) -> &Container {
        &self.containers[self.act]
    }

    fn act_mut(&mut self) -> &mut Container {
        &mut self.containers[self.act]
    }

    fn walk_in_container(&mut self, f: &impl Fn(&mut Container) -> bool) -> bool {
        if f(self.act_mut()) {
            Container::actualize_layout(self);
            match self.act_mut().actual_mut() {
                Some(widget) => widget.focus() || self.walk_in_container(f),
                None => true,
            }
        } else {
            false
        }
    }

    /// Change the focus within the layout.
    ///
    /// # Parameters
    ///
    /// - `next`: An `Option<RcCon>` representing the new container to focus.
    fn change_focus(&mut self, direction: &Direction, f: &impl Fn(&mut Container) -> bool) -> bool {
        log::trace!(
            "Layout::change_focus: direction {:?}, act {}",
            &direction,
            self.act
        );
        let old = Holder::new(self);
        while *self.act().get_direction() != *direction {
            match self.act().parent {
                Some(index) => self.act = index,
                None => return false,
            }
        }
        if f(self.act_mut()) {
            Container::actualize_layout(self);
            if match self.act_mut().actual_mut() {
                Some(widget) => widget.focus() || self.walk_in_container(f),
                None => true,
            } {
                old.unfocus(self);
                true
            } else {
                log::trace!(
                    "Revert to cont: {}, widget: {}",
                    old.container,
                    old.widgets[old.container]
                );
                old.set_old_back(self);
                false
            }
        } else {
            match self.act().parent {
                // check if there is upper container that can handle change
                Some(index) => {
                    self.act = index;
                    if self.change_focus(direction, f) {
                        old.unfocus(self);
                        true
                    } else {
                        old.set_old_back(self);
                        false
                    }
                }
                None => {
                    old.set_old_back(self);
                    false
                }
            }
        }
    }

    /// This method moves the focus to the container or widget to the `Site`
    /// of the currently focused element within the layout.
    fn move_focus(&mut self, direction: Direction, function: fn(&mut Container) -> bool) -> bool {
        let ret = self.change_focus(&direction, &function);
        Container::actualize_layout(self);
        log::debug!(
            "Moved: {ret}, act widget: {}, container: {}, position: {}",
            self.get_active_widget(),
            self.act,
            self.act().get_index(),
        );
        ret
    }

    /// Move the focus to the left.
    pub fn left(&mut self) -> bool {
        self.move_focus(Direction::Horizontal, Container::previous_item)
    }

    /// Move the focus to the right.
    pub fn right(&mut self) -> bool {
        self.move_focus(Direction::Horizontal, Container::next_item)
    }

    /// Move the focus upwards.
    pub fn up(&mut self) -> bool {
        self.move_focus(Direction::Vertical, Container::previous_item)
    }

    /// Move the focus downwards.
    pub fn down(&mut self) -> bool {
        self.move_focus(Direction::Vertical, Container::next_item)
    }

    /// Handle a key event.
    ///
    /// This method is used to handle key events within the layout. It passes the key event to the
    /// currently focused widget or container for processing.
    ///
    /// # Parameters
    ///
    /// - `event`: A reference to the `KeyEvent` to be handled.
    pub fn handle_key(&mut self, event: &KeyEvent) -> bool {
        self.act_mut()
            .actual_mut()
            .expect("Actual is not widget")
            .handle_key(event)
    }

    pub fn get_active_widget(&self) -> WidgetType {
        self.act().get_active_type().expect("Actual is not widget")
    }

    fn find_widget(
        &mut self,
        find_functor: impl Fn(&&mut Widget) -> bool,
        process_widget: impl Fn(&mut Widget),
    ) {
        let cont_act_index = self.act().get_index();
        let indexes = match self
            .containers
            .iter_mut()
            .enumerate()
            .flat_map(|(layout_index, container)| {
                container
                    .get_widgets_mut()
                    .into_iter()
                    .enumerate()
                    .map(move |(widget_index, widget)| (layout_index, widget_index, widget))
            })
            .find(|(_, _, w)| find_functor(w))
        {
            Some((layout_index, cont_index, widget)) => {
                process_widget(widget);
                if self.act == layout_index && cont_act_index == cont_index {
                    None
                } else if widget.focus() {
                    Some((layout_index, cont_index))
                } else {
                    None
                }
            }
            None => None,
        };

        if let Some((layout_index, cont_index)) = indexes {
            if let Some(w) = self.act_mut().actual_mut() {
                w.unfocus()
            }
            self.act = layout_index;
            self.act_mut().set_index(cont_index);
            Container::actualize_parents(self);
        }
    }

    pub fn click(&mut self, column: u16, row: u16) {
        log::debug!("Click on column {column}, row {row}");
        self.find_widget(
            |w| {
                let chunk = &w.get_base().chunk;
                let x = chunk.x < column && column < chunk.x + chunk.width;
                let y = chunk.y < row && row < chunk.y + chunk.height;
                x && y
            },
            |w| w.click(column.into(), row.into()),
        );
    }

    pub fn select_widget(&mut self, widget_type: WidgetType) {
        log::debug!("Select widget {widget_type}");
        self.find_widget(|w| w.widget_type() == widget_type, |_| {});
    }

    pub fn search(&mut self, to_search: String) {
        log::trace!("search to_search={to_search}");
        match self.act_mut().actual_mut() {
            Some(w) => w.search_event(to_search),
            None => panic!("Actual to search is not a widget"),
        }
    }

    pub fn clean_search(&mut self) {
        log::trace!("clean_search");
        match self.act_mut().actual_mut() {
            Some(w) => w.clear_search(),
            None => panic!("Actual to search is not a widget"),
        }
    }
}

impl Render for Layout {
    fn render(&self, f: &mut Frame) {
        self.containers[0].render(f, &self.containers);
    }

    fn unfocus(&mut self) {
        match self.act_mut().actual_mut() {
            Some(w) => w.unfocus(),
            None => panic!("Actual to unfocus is not a widget"),
        }
    }

    fn focus(&mut self) -> bool {
        match self.act_mut().actual_mut() {
            Some(w) => w.focus(),
            None => panic!("Actual to focus is not a widget"),
        }
    }

    fn update_chunk(&mut self, chunk: Rect) {
        Container::update_chunk(chunk, &mut self.containers, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_layout() -> Layout {
        let mock_layout = r#"
        [
            Direction: Horizontal,
            Size: 50%,
            [
                List: 50%,
                Preview,
            ],
            [ Direction: Vertical,
              Done,
              [ 
                Contexts,
                Projects,
              ],
            ],
        ]
        "#;
        Layout::from_str(
            mock_layout,
            Arc::new(Mutex::new(ToDo::default())),
            &Config::default(),
        )
        .unwrap()
    }

    #[test]
    fn test_basic_movement() -> Result<()> {
        let mut l = mock_layout();
        assert_eq!(l.get_active_widget(), WidgetType::List);

        assert!(l.right());
        assert_eq!(l.get_active_widget(), WidgetType::Done);
        assert!(l.left());
        assert_eq!(l.get_active_widget(), WidgetType::List);
        assert!(l.right());
        assert_eq!(l.get_active_widget(), WidgetType::Done);
        assert!(!l.right());
        assert_eq!(l.get_active_widget(), WidgetType::Done);
        assert!(l.down());
        assert_eq!(l.get_active_widget(), WidgetType::Context);
        assert!(l.right());
        assert_eq!(l.get_active_widget(), WidgetType::Project);
        assert!(!l.down());
        assert_eq!(l.get_active_widget(), WidgetType::Project);
        assert!(l.left());
        assert_eq!(l.get_active_widget(), WidgetType::Context);
        assert!(l.left());
        assert_eq!(l.get_active_widget(), WidgetType::List);
        assert!(l.right());
        assert_eq!(l.get_active_widget(), WidgetType::Context);
        assert!(l.left());
        assert_eq!(l.get_active_widget(), WidgetType::List);
        assert!(!l.up());
        assert_eq!(l.get_active_widget(), WidgetType::List);

        Ok(())
    }

    #[test]
    fn test_from_string() -> Result<()> {
        let str_layout = r#"
            [
              dIrEcTiOn:HoRiZoNtAl,
              Size: 50%,
              List: 50%,
              [
                Done,
                Hashtags: 50%,
              ],
              Projects: 50%,
            ]
        "#;

        let mut layout = Layout::from_str(
            str_layout,
            Arc::new(Mutex::new(ToDo::default())),
            &Config::default(),
        )?;
        assert_eq!(layout.containers.len(), 2);

        assert_eq!(*layout.containers[0].get_direction(), Direction::Horizontal);
        assert_eq!(layout.containers[0].parent, None);
        while layout.containers[0].previous_item() {}
        assert_eq!(
            layout.containers[0].get_active_type(),
            Some(WidgetType::List)
        );
        assert!(layout.containers[0].next_item());
        assert_eq!(layout.containers[0].get_active_type(), None);
        assert!(layout.containers[0].next_item());
        assert_eq!(
            layout.containers[0].get_active_type(),
            Some(WidgetType::Project)
        );
        assert!(!layout.containers[0].next_item());

        assert_eq!(*layout.containers[1].get_direction(), Direction::Vertical);
        assert_eq!(layout.containers[1].parent, Some(0));
        while layout.containers[1].previous_item() {}
        assert_eq!(
            layout.containers[1].get_active_type(),
            Some(WidgetType::Done)
        );
        assert!(layout.containers[1].next_item());
        assert_eq!(
            layout.containers[1].get_active_type(),
            Some(WidgetType::Hashtag)
        );
        assert!(!layout.containers[1].next_item());

        Ok(())
    }
}
