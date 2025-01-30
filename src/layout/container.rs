use crate::todo::ToDo;

use super::{render_trait::Render, widget::widget_type::WidgetType, Layout, Widget};
use tui::{
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

    pub fn add_cont(&mut self, container_index: usize) {
        self.items.push(It::Cont(container_index));
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
        self.layout = self.layout.clone().direction(direction);
    }

    pub fn get_direction(&self) -> &Direction {
        &self.direction
    }

    pub fn set_constraints(&mut self, constraints: Vec<Constraint>) {
        self.layout = self.layout.clone().constraints(constraints);
    }

    pub fn get_index(&self) -> usize {
        self.act_index
    }

    pub fn set_index(&mut self, index: usize) -> bool {
        if self.items.len() > index {
            self.act_index = index;
            true
        } else {
            false
        }
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
    /// A result containing a reference to the active `Widget` or a `None`
    /// if the active item is not a widget.
    pub fn actual(&self) -> Option<&Widget> {
        self.get_widget(self.act_index)
    }

    /// Returns a mutable reference to the currently active item within the container.
    ///
    /// # Returns
    ///
    /// A result containing a mutable reference to the active `Widget` or a `None`
    /// if the active item is not a widget.
    pub fn actual_mut(&mut self) -> Option<&mut Widget> {
        self.get_widget_mut(self.act_index)
    }

    // If layouts actual item points to container whose actual points to container,
    // actualize it and change actual layouts actual to container that really points
    // to widget.
    pub fn actualize_layout(layout: &mut Layout) {
        fn find_actual(layout: &Layout) -> usize {
            if let It::Cont(mut index) = layout.act().items[layout.act().act_index] {
                while let It::Cont(cont) =
                    &layout.containers[index].items[layout.containers[index].act_index]
                {
                    index = *cont;
                }
                index
            } else {
                layout.act
            }
        }
        layout.act = find_actual(layout);
    }

    pub fn actualize_parents(layout: &mut Layout) {
        let mut child_index = layout.act;
        while let Some(parent) = layout.containers[child_index].parent {
            let cont = &layout.containers[parent];
            layout.containers[parent].act_index = cont
                .items
                .iter()
                .position(|w| {
                    if let It::Cont(cont) = w {
                        std::ptr::eq(&layout.containers[*cont], &layout.containers[child_index])
                    } else {
                        false
                    }
                })
                .expect("Child should be in parent container.");
            child_index = parent;
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
        log::trace!("Next item {}", self.act_index);
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
        log::trace!("Prev item {}", self.act_index);
        if self.act_index > 0 {
            self.act_index -= 1;
            true
        } else {
            false
        }
    }

    pub fn get_active_type(&self) -> Option<WidgetType> {
        Some(self.actual()?.widget_type())
    }

    pub fn render(&self, f: &mut Frame, containers: &Vec<Self>, todo: &ToDo) {
        self.items.iter().for_each(|cont| match cont {
            It::Cont(index) => containers[*index].render(f, containers, todo),
            It::Item(widget) => widget.render(f, todo),
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

    pub fn get_widgets_mut(&mut self) -> impl IntoIterator<Item = &mut Widget> {
        self.items.iter_mut().filter_map(|item| {
            if let It::Item(w) = item {
                Some(w)
            } else {
                None
            }
        })
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
    use crate::{config::Config, layout::widget::State, todo::ToDo, Result};
    use WidgetType::*;

    fn testing_layout() -> Layout {
        Layout::from_str(
            r#"
            [
                Direction: Horizontal,
                Size: 30%,
                List: 50%,
                [
                    Direction: Vertical,
                    Done: 50%,
                    Projects: 50%,
                ],
            ]
            "#,
            &ToDo::default(),
            &Config::default(),
        )
        .unwrap()
    }

    fn check_active(layout: &Layout, widget_type: WidgetType) {
        match layout.act().get_active_type() {
            Some(active) if active == widget_type => {}
            Some(active) => panic!("Active widget must be {:?} not {:?}.", widget_type, active),
            None => panic!("Active item is not widget"),
        }
    }

    #[test]
    fn test_selecting_widget() -> Result<()> {
        let mut layout = testing_layout();
        let mut check = |widget_type| -> Result<()> {
            layout.select_widget(widget_type, &ToDo::default());
            check_active(&layout, widget_type);
            Ok(())
        };

        check(List)?;
        check(Done)?;
        check(Project)?;

        // If Context is not find it is not set.
        layout.select_widget(Context, &ToDo::default());
        check_active(&layout, Project);

        Ok(())
    }

    #[test]
    fn test_next_item() -> Result<()> {
        let mut layout = testing_layout();

        // Test next widget in child container.
        layout.select_widget(List, &ToDo::default());
        assert!(layout.act_mut().next_item());
        Container::actualize_layout(&mut layout);
        check_active(&layout, Done);

        // Test next widget in same container.
        layout.select_widget(Done, &ToDo::default());
        assert!(layout.act_mut().next_item());
        Container::actualize_layout(&mut layout);
        check_active(&layout, Project);

        // Test next in container have not default value
        layout.select_widget(List, &ToDo::default());
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
    fn test_previous_item() -> Result<()> {
        let mut layout = testing_layout();

        // Test previous widget in same container.
        layout.select_widget(Project, &ToDo::default());
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
        // assert_eq!(0, count_widgets(0));
        assert_eq!(1, count_widgets(0));
        check_chunk(0, 0, Rect::new(0, 0, 10, 20));
        check_chunk(1, 0, Rect::new(10, 0, 6, 10));
        assert_eq!(2, count_widgets(1));
        check_chunk(1, 1, Rect::new(10, 10, 6, 10));
    }
}
