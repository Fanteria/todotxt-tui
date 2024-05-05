use super::{
    render_trait::Render,
    widget::widget_type::WidgetType, 
    Widget,
    Layout,
};
use crate::error::{ToDoError, ToDoRes};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout as TuiLayout, Rect},
    Frame,
};

#[derive(Debug)]
enum It {
    Cont(usize),
    Item(Widget),
}

/// Represents a container that can hold widgets and other containers.
///
/// A `Container` is a component that can hold a collection of `Item`s, which can be either
/// widgets or nested containers. It provides methods for rendering, focusing, and updating
/// the contained items.
#[derive(Debug)]
pub struct Container {
    items: Vec<It>,
    layout: TuiLayout,
    direction: Direction,
    pub parent: Option<usize>,
    act_index: usize,
}

impl Container {
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

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn get_index(&self) -> usize {
        self.act_index
    }

    pub fn get_widget(&self, index: usize) -> Option<&Widget> {
        match &self.items[index] {
            It::Item(w) => Some(w),
            It::Cont(_) => None,
        }
    }

    pub fn get_widget_mut(&mut self, index: usize) -> Option<&mut Widget> {
        match &mut self.items[index] {
            It::Item(w) => Some(w),
            It::Cont(_) => None,
        }
    }

    /// Returns a reference to the currently active item within the container.
    ///
    /// # Returns
    ///
    /// A result containing a reference to the active `Widget` or an error if the active item
    /// is not a widget.
    #[allow(dead_code)]
    pub fn actual(&self) -> Option<&Widget> {
        self.get_widget(self.act_index)
    }

    /// Returns a mutable reference to the currently active item within the container.
    ///
    /// # Returns
    ///
    /// A result containing a mutable reference to the active `Widget` or an error if the active
    /// item is not a widget.
    pub fn actual_mut(&mut self) -> Option<&mut Widget> {
        self.get_widget_mut(self.act_index)
    }

    // TODO Change to actual widget index
    pub fn actualize_layout(layout: &mut Layout) {
        if let It::Cont(mut index) = layout.act().items[layout.act().act_index] {
            while let It::Cont(cont) =
                &layout.containers[index].items[layout.containers[index].act_index]
            {
                index = *cont;
            }
            layout.act = index;
        }
        layout.act_mut().actual_mut().unwrap().focus();
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
        let mut index_item = 0;
        let (index_container, _) = layout
            .containers
            .iter()
            .enumerate()
            .find(|(_i_cont, cont)| {
                cont.items
                    .iter()
                    .enumerate()
                    .any(|(i_item, item)| match item {
                        It::Item(item) if item.widget_type() == widget_type => {
                            index_item = i_item;
                            true
                        }
                        _ => false,
                    })
            })
            .ok_or(ToDoError::WidgetDoesNotExist)?;
        layout.containers[index_container].act_index = index_item;
        layout.act = index_container;

        // Reproduce path back to root.
        let mut index_container = index_container;
        while let Some(index_parent) = layout.containers[index_container].parent {
            layout.containers[index_parent].act_index = index_container;
            index_container = index_parent;
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

    pub fn update_chunk(chunk: Rect, containers: &mut Vec<Self>, index: usize) {
        let chunks = containers[index].layout.split(chunk);
        for i in 0..containers[index].items.len() {
            let index = match &mut containers[index].items[i] {
                It::Cont(index) => *index,
                It::Item(widget) => {
                    widget.update_chunk(chunks[i]);
                    continue;
                }
            };
            Self::update_chunk(chunks[i], containers, index);
        }
    }
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
    use crate::{config::Config, layout::widget::State, todo::ToDo};
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
        let mut cont = Container::default();
        cont.parent = Some(index);
        cont.set_direction(Horizontal);
        cont.set_constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
        // Left widget
        cont.add_widget(Widget::new(WidgetType::List, todo.clone(), &Config::default()).unwrap());
        let index = Container::add_container(&mut containers, cont);

        // Right container
        let mut cont = Container::default();
        cont.parent = Some(index);
        cont.set_direction(Vertical);
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

    #[test]
    fn test_update_chunk() {
        let mut layout = testing_layout();
        layout.update_chunk(Rect::new(0, 0, 20, 20));
        let count_widgets = |index: usize| -> usize {
            layout.containers[index]
                .items
                .iter()
                .filter(|item| match item {
                    It::Cont(_) => false,
                    It::Item(_) => true,
                })
                .count()
        };
        let check_chunk = |c_index: usize, i_index: usize, rect| {
            match &layout.containers[c_index].items[i_index] {
                It::Cont(_) => panic!("Cointainer does not hold widget"),
                It::Item(widget) => assert_eq!(widget.get_base().chunk, rect),
            };
        };
        assert_eq!(0, count_widgets(0));
        assert_eq!(1, count_widgets(1));
        check_chunk(1, 0, Rect::new(0, 0, 10, 20));
        assert_eq!(2, count_widgets(2));
        check_chunk(2, 0, Rect::new(10, 0, 10, 10));
        check_chunk(2, 1, Rect::new(10, 10, 10, 10));
    }
}
