mod holder;
mod item;

use super::render_trait::Render;
use super::widget::{widget_type::WidgetType, Widget};
use super::Layout;
use crate::error::{ToDoError, ToDoRes};
#[allow(unused_imports)]
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout as TuiLayout, Rect},
    Frame,
};

pub enum It {
    Cont(usize),
    Item(Widget),
}

/// Represents a container that can hold widgets and other containers.
///
/// A `Container` is a component that can hold a collection of `Item`s, which can be either
/// widgets or nested containers. It provides methods for rendering, focusing, and updating
/// the contained items.
pub struct Container {
    items: Vec<It>,
    layout: TuiLayout,
    direction: Direction,
    pub parent: Option<usize>,
    act_index: usize,
}

impl Container {
    /// Creates a new container with the given items, constraints, direction, and parent.
    ///
    /// # Parameters
    ///
    /// - `items`: A vector of items (widgets or containers) to be placed within the container.
    /// - `constraints`: A vector of constraints defining how items should be laid out within
    ///   the container.
    /// - `direction`: The layout direction of the container (Vertical or Horizontal).
    /// - `parent`: An optional reference to the parent container, if this container is nested
    ///   within another container.
    ///
    /// # Returns
    ///
    /// A reference-counted (`Rc`) reference to the newly created `Container`.
    // pub fn new(
    //     constraints: Vec<Constraint>,
    //     direction: Direction,
    //     parent: Option<usize>,
    // ) -> Container {
    //     Container {
    //         layout: TuiLayout::default()
    //             .direction(direction.clone())
    //             .constraints(constraints),
    //         direction,
    //         parent,
    //         ..Container::default()
    //     }
    // }

    pub fn add_widget(&mut self, widget: Widget) {
        self.items.push(It::Item(widget));
    }

    pub fn add_container(containers: &mut Vec<Self>, container: Container) -> usize {
        let index = containers.len();
        // check if container have parent
        if let Some(parent_index) = container.parent {
            containers[parent_index].items.push(It::Cont(index));
        }
        containers.push(container);
        index
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction.clone();
        self.layout = self.layout.clone().direction(direction);
    }

    #[allow(dead_code)]
    pub fn get_direction(&self) -> &Direction {
        &self.direction
    }

    pub fn set_constraints(&mut self, constraints: Vec<Constraint>) {
        self.layout = self.layout.clone().constraints(constraints);
    }

    /// Returns a reference to the currently active item within the container.
    ///
    /// # Returns
    ///
    /// A result containing a reference to the active `Widget` or an error if the active item
    /// is not a widget.
    #[allow(dead_code)]
    pub fn actual(&self) -> Option<&Widget> {
        match &self.items[self.act_index] {
            It::Item(w) => Some(w),
            It::Cont(_) => None,
        }
    }

    /// Returns a mutable reference to the currently active item within the container.
    ///
    /// # Returns
    ///
    /// A result containing a mutable reference to the active `Widget` or an error if the active
    /// item is not a widget.
    pub fn actual_mut(&mut self) -> Option<&mut Widget> {
        match &mut self.items[self.act_index] {
            It::Item(w) => Some(w),
            It::Cont(_) => None,
        }
    }

    // TODO Change to actual widget index
    pub fn actualize_layout(layout: &mut Layout) {
        if let It::Cont(mut index) = layout.act().items[layout.act().act_index] {
            loop {
                match &layout.containers[index].items[layout.containers[index].act_index] {
                    It::Cont(cont) => index = *cont,
                    It::Item(_) => break,
                }
            }
            layout.act = index;
        }
    }

    /// Attempts to select the next item within the container.
    ///
    /// # Parameters
    ///
    /// - `container`: A reference-counted (Rc) reference to the container to navigate within.
    ///
    /// # Returns
    ///
    /// An option containing either an updated reference to the container with the next item
    /// as the active item, or `None` if there is no next item to select within the container.
    pub fn next_item(&mut self) -> bool {
        if self.items.len() > self.act_index + 1 {
            self.act_index += 1;
            true
        } else {
            false
        }
    }

    /// Attempts to select the previous item within the container.
    ///
    /// # Parameters
    ///
    /// - `container`: A reference-counted (Rc) reference to the container to navigate within.
    ///
    /// # Returns
    ///
    /// An option containing either an updated reference to the container with the previous item
    /// as the active item, or `None` if there is no previous item to select within the container.
    ///
    pub fn previous_item(&mut self) -> bool {
        if self.act_index > 0 {
            self.act_index -= 1;
            true
        } else {
            false
        }
    }

    /// Finds and selects a specific widget type within the container.
    ///
    /// # Parameters
    ///
    /// - `container`: A reference-counted (Rc) reference to the container to search within.
    /// - `widget_type`: The `WidgetType` enum variant representing the target widget type.
    ///
    /// # Returns
    ///
    /// A result containing either an updated reference to the container with the selected widget
    /// type as the active item, or an error if the widget type is not found within the container.
    #[allow(dead_code)]
    pub fn select_widget(layout: &mut Layout, widget_type: WidgetType) -> ToDoRes<()> {
        // Set everything to first item
        layout
            .containers
            .iter_mut()
            .for_each(|cont| cont.act_index = 0);
        layout.act = 0;
        Container::actualize_layout(layout);

        loop {
            if layout.act().get_active_type() == Some(widget_type) {
                break;
            }
            let mut next = layout.act_mut().next_item();
            while !next {
                match layout.act().parent {
                    Some(index) => {
                        layout.act = index;
                    }
                    None => return Err(ToDoError::WidgetDoesNotExist),
                }
                next = layout.act_mut().next_item();
            }
            Container::actualize_layout(layout)
        }
        Ok(())
    }

    pub fn get_active_type(&self) -> Option<WidgetType> {
        Some(self.actual()?.widget_type())
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, containers: &Vec<Self>) {
        self.items.iter().for_each(|cont| match cont {
            It::Cont(index) => containers[*index].render(f, containers),
            It::Item(widget) => widget.render(f),
        });
    }

    // pub fn update_chunk(&mut self, chunk: Rect, containers: &mut Vec<Self>) {
    //     self.items.iter_mut().for_each(|cont| match cont {
    //         It::Cont(index) => containers[*index].update_chunk(chunk, &mut containers), // TODO may be splitted
    //         It::Item(widget) => widget.update_chunk(chunk),
    //     });
    // }
}

impl Default for Container {
    fn default() -> Self {
        Container {
            items: Vec::new(),
            layout: TuiLayout::default(),
            direction: Direction::Vertical,
            parent: None,
            act_index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Layout;
    use super::*;
    use crate::{config::Config, todo::ToDo};
    use std::sync::{Arc, Mutex};
    use tui::layout::Direction::*;
    use WidgetType::*;

    fn testing_layout() -> Layout {
        let todo = Arc::new(Mutex::new(ToDo::default()));

        // Main container
        let mut containers: Vec<Container> = Vec::new();
        let index = Container::add_container(&mut containers, Container::default());
        containers[index].set_direction(Vertical);
        containers[index].set_constraints(vec![Constraint::Percentage(30)]);

        // Holder container
        let mut cont = Container {
            parent: Some(index),
            direction: Horizontal,
            ..Container::default()
        };
        cont.set_constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
        // Left widget
        cont.add_widget(Widget::new(WidgetType::List, todo.clone(), &Config::default()).unwrap());
        let index = Container::add_container(&mut containers, cont);

        // Right container
        let mut cont = Container {
            parent: Some(index),
            direction: Vertical,
            ..Container::default()
        };
        cont.set_constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
        cont.add_widget(Widget::new(WidgetType::Done, todo.clone(), &Config::default()).unwrap());
        cont.add_widget(Widget::new(WidgetType::Project, todo, &Config::default()).unwrap());
        let index = Container::add_container(&mut containers, cont);

        Layout {
            containers,
            act: index,
        }
    }

    fn check_active(layout: &Layout, widget_type: WidgetType) {
        match layout.act().get_active_type() {
            Some(active) if active == widget_type => {}
            Some(active) => panic!("Active widget must be {:?} not {:?}.", widget_type, active),
            None => panic!("Active item is not widget"),
        }
    }

    #[test]
    fn test_selecting_widget() -> ToDoRes<()> {
        let mut layout = testing_layout();
        let mut check = |widget_type| -> ToDoRes<()> {
            Container::select_widget(&mut layout, widget_type)?;
            check_active(&layout, widget_type);
            Ok(())
        };

        check(List)?;
        check(Done)?;
        check(Project)?;
        assert!(
            check(Context).is_err(),
            "Widget with type Context is not in container."
        );

        Ok(())
    }

    #[test]
    fn test_next_item() -> ToDoRes<()> {
        let mut layout = testing_layout();

        // Test next widget in child container.
        Container::select_widget(&mut layout, List)?;
        assert!(layout.act_mut().next_item());
        Container::actualize_layout(&mut layout);
        check_active(&layout, Done);

        // Test next widget in same container.
        Container::select_widget(&mut layout, Done)?;
        assert!(layout.act_mut().next_item());
        Container::actualize_layout(&mut layout);
        check_active(&layout, Project);

        // Test next in container have not default value
        Container::select_widget(&mut layout, List)?;
        assert!(layout.act_mut().next_item());
        Container::actualize_layout(&mut layout);
        check_active(&layout, Project);

        // Test return value if there is no next item
        assert!(!layout.act_mut().next_item());
        Container::actualize_layout(&mut layout);
        assert!(!layout.act_mut().next_item());
        Container::actualize_layout(&mut layout);
        assert!(!layout.act_mut().next_item());
        Container::actualize_layout(&mut layout);
        assert_eq!(layout.act().act_index, 1);
        check_active(&layout, Project);

        Ok(())
    }

    #[test]
    fn test_previous_item() -> ToDoRes<()> {
        let mut layout = testing_layout();

        // Test previous widget in same container.
        Container::select_widget(&mut layout, Project)?;
        assert!(layout.act_mut().previous_item());
        Container::actualize_layout(&mut layout);

        // Test return value if there is no previous item
        assert!(!layout.act_mut().previous_item());
        Container::actualize_layout(&mut layout);
        assert!(!layout.act_mut().previous_item());
        Container::actualize_layout(&mut layout);
        assert!(!layout.act_mut().previous_item());
        Container::actualize_layout(&mut layout);
        assert_eq!(layout.act().act_index, 0);
        check_active(&layout, Done);

        Ok(())
    }
}
