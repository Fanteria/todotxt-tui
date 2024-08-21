mod container;
mod render_trait;
pub mod widget;

use crate::{
    config::Config, layout::widget::State, todo::ToDo, ui::HandleEvent, Result, ToDoError,
};
use container::Container;
use crossterm::event::KeyEvent;
use std::{fmt::Debug, sync::Arc, sync::Mutex};
use widget::{widget_type::WidgetType, Widget};

pub use render_trait::Render;

use std::str::FromStr;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Rect},
    Frame,
};

// Define separators
const ITEM_SEPARATOR: char = ',';
const ARG_SEPARATOR: char = ':';
const START_CONTAINER: char = '[';
const END_CONTAINER: char = ']';

const LEFT: Site = Site {
    direction: Direction::Horizontal,
    function: Container::previous_item,
};
const RIGHT: Site = Site {
    direction: Direction::Horizontal,
    function: Container::next_item,
};
const UP: Site = Site {
    direction: Direction::Vertical,
    function: Container::previous_item,
};
const DOWN: Site = Site {
    direction: Direction::Vertical,
    function: Container::next_item,
};

struct Site {
    direction: Direction,
    function: fn(&mut Container) -> bool,
}

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
    /// Parse and convert a string value to a `Constraint`.
    ///
    /// # Parameters
    ///
    /// - `value`: A string slice representing the layout constraint.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the converted `Constraint` or an error if parsing fails.
    fn value_from_string(value: Option<&str>) -> Result<Constraint> {
        Ok(match value {
            Some(value) => match value.find('%') {
                Some(i) if i + 1 < value.len() => {
                    return Err(ToDoError::ParseUnknownValue(value.to_string()))
                }
                Some(i) => Constraint::Percentage(value[..i].parse()?),
                None => Constraint::Length(value.parse()?),
            },
            None => Constraint::Percentage(50),
        })
    }

    fn process_item(
        item: &str,
        container: &mut Container,
        data: Arc<Mutex<ToDo>>,
        config: &Config,
    ) -> Result<Option<Constraint>> {
        log::trace!("Process item: {item}");
        let s = item.to_lowercase();
        let x: Vec<&str> = s.splitn(2, ARG_SEPARATOR).map(|s| s.trim()).collect();
        let x = (x[0], if x.len() > 1 { Some(x[1]) } else { None });
        match x.0 {
            "direction" => {
                match x.1 {
                    None | Some("vertical") => container.set_direction(Direction::Vertical),
                    Some("horizontal") => container.set_direction(Direction::Horizontal),
                    Some(direction) => {
                        return Err(ToDoError::ParseInvalidDirection(direction.to_owned()))
                    }
                }
                Ok(None)
            }
            "size" => Ok(Some(Self::value_from_string(x.1)?)),
            _ => {
                container.add_widget(Widget::new(
                    WidgetType::from_str(x.0)?,
                    data.clone(),
                    config,
                )?);
                Ok(Some(Self::value_from_string(x.1)?))
            }
        }
    }

    /// Create a new `Layout` from a template string.
    ///
    /// This function parses a template string and creates a new `Layout` instance based on the
    /// specified template. The template string defines the layout of the user interface, including
    /// the arrangement of containers and widgets.
    ///
    /// # Parameters
    ///
    /// - `template`: A string containing the layout template.
    /// - `data`: An `Arc<Mutex<ToDo>>` representing the shared to-do data.
    ///
    /// # Returns
    ///
    /// A `Result<Self>` result containing the created `Layout` if successful, or an error if
    /// parsing fails.
    pub fn from_str(template: &str, data: Arc<Mutex<ToDo>>, config: &Config) -> Result<Self> {
        // Find first '[' and move start of template to it (start of first container)
        let index = match template.find('[') {
            Some(i) => i,
            None => return Err(ToDoError::ParseNotStart),
        };
        let template = &template[index + 1..];
        log::debug!("Layout from str: {}", template);

        let mut string = String::new();

        let mut constraints_stack: Vec<Vec<Constraint>> = Vec::new();
        constraints_stack.push(Vec::new());
        let mut containers: Vec<Container> = Vec::new();
        let mut layout = Layout {
            act: Container::add_container(&mut containers, Container::default()),
            containers,
        };

        for ch in template.chars() {
            match ch {
                START_CONTAINER => {
                    if !string.is_empty() {
                        return Err(ToDoError::ParseUnknowBeforeContainer(string));
                    }
                    if layout.act().item_count() >= constraints_stack.last().unwrap().len() {
                        constraints_stack
                            .last_mut()
                            .unwrap()
                            .push(Constraint::Percentage(50));
                    }
                    let mut cont = Container::default();
                    cont.parent = Some(layout.act);
                    cont.set_direction(match layout.act().get_direction() {
                        Direction::Horizontal => Direction::Vertical,
                        Direction::Vertical => Direction::Horizontal,
                    });
                    layout.act = Container::add_container(&mut layout.containers, cont);
                    constraints_stack.push(Vec::new());
                }
                END_CONTAINER => {
                    log::trace!(
                        "Act: {}, Constraints: {:?}",
                        layout.act,
                        constraints_stack.last()
                    );
                    layout
                        .act_mut()
                        .set_constraints(constraints_stack.pop().unwrap());
                    layout.act = match layout.act().parent {
                        Some(parent) => parent,
                        // We are at root. Return created layout.
                        None => {
                            Container::actualize_layout(&mut layout);
                            layout.act_mut().actual_mut().unwrap().focus();
                            return Ok(layout);
                        }
                    };
                    string.clear();
                }
                ITEM_SEPARATOR => {
                    // Skip leading ITEM_SEPARATOR
                    if !string.is_empty() {
                        if let Some(constrain) =
                            Self::process_item(&string, layout.act_mut(), data.clone(), config)?
                        {
                            constraints_stack.last_mut().unwrap().push(constrain);
                        }
                        string.clear();
                    }
                }
                ' ' => {}
                '\n' => {}
                _ => string.push(ch),
            };
        }
        Err(ToDoError::ParseNotEnd)
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
    fn move_focus(&mut self, site: &Site) -> bool {
        let ret = self.change_focus(&site.direction, &site.function);
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
        self.move_focus(&LEFT)
    }

    /// Move the focus to the right.
    pub fn right(&mut self) -> bool {
        self.move_focus(&RIGHT)
    }

    /// Move the focus upwards.
    pub fn up(&mut self) -> bool {
        self.move_focus(&UP)
    }

    /// Move the focus downwards.
    pub fn down(&mut self) -> bool {
        self.move_focus(&DOWN)
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
        match self.act_mut().actual_mut() {
            Some(widget) => widget.handle_key(&event.code),
            None => panic!("Actual is not widget"),
        }
    }

    pub fn get_active_widget(&self) -> WidgetType {
        match self.act().get_active_type() {
            Some(widget_type) => widget_type,
            None => panic!("Actual is not widget"),
        }
    }

    pub fn click(&mut self, column: u16, row: u16) {
        log::debug!("Click on column {column}, row {row}");
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
            .find(|(_, _, w)| {
                let chunk = &w.get_base().chunk;
                let x = chunk.x < column && column < chunk.x + chunk.width;
                let y = chunk.y < row && row < chunk.y + chunk.height;
                x && y
            }) {
            Some((layout_index, cont_index, widget)) => {
                widget.click(column.into(), row.into());
                if self.act == layout_index && cont_act_index == cont_index {
                    None
                } else if widget.focus() {
                    Some((layout_index, cont_index))
                } else {
                    None
                }
            }
            None => {
                log::error!("There is no chunk laying on column {column}, row {row}");
                None
            }
        };

        if let Some((layout_index, cont_index)) = indexes {
            if let Some(w) = self.act_mut().actual_mut() {
                w.unfocus()
            }
            self.act = layout_index;
            self.act_mut().set_index(cont_index);
            Container::actualize_layout(self);
        }
    }
}

impl Render for Layout {
    fn render<B: Backend>(&self, f: &mut Frame<B>) {
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
            
            Direction: ERROR,
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
