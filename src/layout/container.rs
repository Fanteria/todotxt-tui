mod holder;
mod item;

use self::holder::Holder;
use super::render_trait::Render;
use super::widget::{widget_type::WidgetType, Widget};
use crate::error::{ToDoError, ToDoRes};
use item::IItem;
pub use item::Item;
use std::{cell::RefCell, rc::Rc};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout as TuiLayout, Rect},
    Frame,
};

pub type RcCon = Rc<RefCell<Container>>;

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
    // items: Vec<IItem>,
    items: Vec<It>,
    layout: TuiLayout,
    pub direction: Direction,
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
    pub fn new(
        constraints: Vec<Constraint>,
        direction: Direction,
        parent: Option<usize>,
    ) -> Container {
        Container {
            items: Vec::new(),
            layout: TuiLayout::default()
                .direction(direction.clone())
                .constraints(constraints),
            direction,
            parent,
            act_index: 0,
        }
    }

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
        if self.items.len() < self.act_index + 1 {
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
    pub fn select_widget(
        containers: &Vec<Self>,
        index: usize,
        widget_type: WidgetType,
    ) -> ToDoRes<usize> {
        for (index, item) in containers[index].items.iter().enumerate() {
            match item {
                It::Item(w) => {
                    if w.widget_type() == widget_type {
                        containers[index].act_index = index;
                        return Ok(0);
                    }
                }
                It::Cont(container) => {
                    if let Ok(index) = Container::select_widget(containers, *container, widget_type)
                    {
                        return Ok(index);
                    }
                }
            }
        }
        Err(ToDoError::WidgetDoesNotExist)
    }

    pub fn get_active_type(&self) -> WidgetType {
        match self.actual() {
            Some(w) => w.widget_type(),
            None => panic!("The current item is expected to be a widget."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, todo::ToDo};
    use std::sync::{Arc, Mutex};
    use tui::layout::Direction::*;
    use WidgetType::*;

    fn create_testing_container() -> Vec<Container> {
        let todo = Arc::new(Mutex::new(ToDo::default()));

        // Main container
        let mut containers: Vec<Container> = Vec::new();
        let index = Container::add_container(
            &mut containers,
            Container::new(
                vec![Constraint::Length(3), Constraint::Percentage(30)],
                Vertical,
                None,
            ),
        );

        // Holder container
        let mut cont = Container::new(
            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
            Horizontal,
            Some(index),
        );
        // Left widget
        cont.add_widget(Widget::new(WidgetType::List, todo.clone(), &Config::default()).unwrap());
        let index = Container::add_container(&mut containers, cont);

        // Right container
        let mut cont = Container::new(
            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
            Vertical,
            Some(index),
        );
        cont.add_widget(Widget::new(WidgetType::Done, todo.clone(), &Config::default()).unwrap());
        cont.add_widget(Widget::new(WidgetType::Project, todo, &Config::default()).unwrap());
        Container::add_container(&mut containers, cont);

        // Container::new(
        //     vec![Item::Container(Container::new(
        //         vec![
        //             Item::Widget(list_widget),
        //             Item::Container(Container::new(
        //                 vec![Item::Widget(done_widget), Item::Widget(project_widget)],
        //                 vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        //                 Vertical,
        //                 None,
        //             )),
        //         ],
        //         vec![Constraint::Percentage(50), Constraint::Percentage(50)],
        //         Horizontal,
        //         None,
        //     ))],
        //     vec![Constraint::Length(3), Constraint::Percentage(30)],
        //     Vertical,
        //     None,
        // )
        containers
    }

    fn check_active_test(
        containers: &Vec<Container>,
        active_index: usize,
        widget_type: WidgetType,
    ) {
        let active = containers[active_index].get_active_type();
        if active != widget_type {
            panic!("Active widget must be {:?} not {:?}.", widget_type, active)
        }
    }

    #[test]
    fn test_selecting_widget() -> ToDoRes<()> {
        let containers = create_testing_container();
        let check = |widget_type| -> ToDoRes<()> {
            let index = Container::select_widget(&containers, 0, widget_type)?;
            check_active_test(&containers, index, widget_type);
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
        let c = create_testing_container();

        // Test next widget in child container.
        let index = Container::select_widget(&c, 0, List)?;
        assert!(c[index].next_item());
        check_active_test(&c, index, Done);

        // Test next widget in same container.
        let index = Container::select_widget(&c, 0, Done)?;
        assert!(c[index].next_item());
        check_active_test(&c, index, Project);

        // Test next in container have not default value
        let index = Container::select_widget(&c, 0, List)?;
        assert!(c[index].next_item());
        check_active_test(&c, index, Project);

        // Test return value if there is no next item
        assert!(!c[index].next_item());
        assert!(!c[index].next_item());
        assert!(!c[index].next_item());
        assert_eq!(c[index].act_index, 1);
        check_active_test(&c, index, Project);

        Ok(())
    }

    #[test]
    fn test_previous_item() -> ToDoRes<()> {
        let c = create_testing_container();

        // Test previous widget in same container.
        let index = Container::select_widget(&c, 0, Project)?;
        assert!(c[index].previous_item());

        // Test return value if there is no previous item
        assert!(!c[index].previous_item());
        assert!(!c[index].previous_item());
        assert!(!c[index].previous_item());
        assert_eq!(c[index].act_index, 0);
        check_active_test(&c, index, Done);

        Ok(())
    }
}
