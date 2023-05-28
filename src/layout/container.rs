use std::cell::RefCell;
use std::rc::Rc;

use crate::error::{ErrorToDo, ErrorType};

use super::widget::{Widget, WidgetType};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

type RcCon = Rc<RefCell<Container>>;

pub enum Item {
    Container(RcCon),
    Widget(Holder),
}

pub enum InitItem {
    InitContainer(RcCon),
    InitWidget(Widget),
}

pub struct Holder {
    pub widget: Widget,
    pub parent: RcCon,
}

pub struct Container {
    pub items: Vec<Item>,
    pub layout: Layout,
    pub direction: Direction,
    pub parent: Option<RcCon>,
    pub act_index: usize,
    pub active: bool,
}

#[allow(dead_code)]
impl Container {
    pub fn new(
        items: Vec<InitItem>,
        constraints: Vec<Constraint>,
        direction: Direction,
        parent: Option<RcCon>,
    ) -> RcCon {
        let container = Rc::new(RefCell::new(Container {
            items: Vec::new(),
            layout: Layout::default()
                .direction(direction.clone())
                .constraints(constraints),
            direction,
            parent,
            act_index: 0,
            active: false,
        }));

        for item in items {
            match item {
                InitItem::InitWidget(widget) => {
                    container.borrow_mut().items.push(Item::Widget(Holder {
                        widget,
                        parent: Rc::clone(&container),
                    }));
                }
                InitItem::InitContainer(cont) => {
                    cont.borrow_mut().parent = Some(Rc::clone(&container));
                    container.borrow_mut().items.push(Item::Container(cont))
                }
            }
        }

        container
    }

    pub fn update_chunks(&mut self, chunk: Rect) {
        let chunks = self.layout.split(chunk);
        for (i, item) in self.items.iter_mut().enumerate() {
            match item {
                Item::Widget(holder) => holder.widget.update_chunk(chunks[i]),
                Item::Container(container) => container.borrow_mut().update_chunks(chunks[i]),
            }
        }
    }

    pub fn actual_item(&self) -> &Item {
        &self.items[self.act_index]
    }

    fn update_actual(container: &RcCon) -> RcCon {
        let mut borrow = container.borrow_mut();
        match borrow.actual_item() {
            Item::Widget(_) => {
                borrow.active = true;
                return Rc::clone(container);
            }
            Item::Container(cont) => return Container::update_actual(cont),
        }
    }

    fn change_item(
        container: &RcCon,
        condition: fn(&Container) -> bool,
        change: fn(&mut Container),
    ) -> Option<RcCon> {
        if condition(&container.borrow()) {
            return None;
        }
        change(&mut container.borrow_mut());
        Some(Container::update_actual(container))
    }

    pub fn next_item(container: RcCon) -> Option<RcCon> {
        Container::change_item(
            &container,
            |c| c.act_index + 1 >= c.items.len(),
            |c| c.act_index += 1,
        )
    }

    pub fn previous_item(container: RcCon) -> Option<RcCon> {
        Container::change_item(&container, |c| c.act_index <= 0, |c| c.act_index -= 1)
    }

    pub fn select_widget(container: RcCon, widget_type: WidgetType) -> Result<RcCon, ErrorToDo> {
        let mut borrowed = container.borrow_mut();
        for (index, item) in borrowed.items.iter().enumerate() {
            match item {
                Item::Widget(holder) => {
                    if holder.widget.widget_type == widget_type {
                        borrowed.active = true;
                        borrowed.act_index = index;
                        return Ok(container.clone());
                    }
                }
                Item::Container(container) => {
                    let cont = Container::select_widget(container.clone(), widget_type);
                    if cont.is_ok() {
                        borrowed.active = true;
                        borrowed.act_index = index;
                        return cont;
                    }
                }
            }
        }
        Err(ErrorToDo::new(
            ErrorType::WidgetDoesNotExist,
            "Selected widgent is not in layout",
        ))
    }

    pub fn render_recursive<B>(&self, f: &mut Frame<B>)
    where
        B: Backend,
    {
        for (index, item) in self.items.iter().enumerate() {
            match item {
                Item::Widget(holder) => holder
                    .widget
                    .draw(f, self.active && self.act_index == index),
                Item::Container(container) => container.borrow().render_recursive(f),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::todo::ToDo;

    use super::*;
    use tui::layout::Direction::{Horizontal, Vertical};
    use WidgetType::*;

    fn create_testing_container() -> RcCon {
        let todo = Rc::new(ToDo::new(false));
        let input_widget = Widget::new(WidgetType::Input, "Input", todo.clone());
        let list_widget = Widget::new(WidgetType::List, "List", todo.clone());
        let done_widget = Widget::new(WidgetType::Done, "Done", todo.clone());
        let project_widget = Widget::new(WidgetType::Project, "Project", todo.clone());
        let cnt = Container::new(
            vec![
                InitItem::InitWidget(input_widget),
                InitItem::InitContainer(Container::new(
                    vec![
                        InitItem::InitWidget(list_widget),
                        InitItem::InitContainer(Container::new(
                            vec![
                                InitItem::InitWidget(done_widget),
                                InitItem::InitWidget(project_widget),
                            ],
                            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
                            Vertical,
                            None,
                        )),
                    ],
                    vec![Constraint::Percentage(50), Constraint::Percentage(50)],
                    Horizontal,
                    None,
                )),
            ],
            vec![Constraint::Length(3), Constraint::Percentage(30)],
            Vertical,
            None,
        );
        cnt
    }

    fn check_active(container: &RcCon, widget_type: WidgetType) {
        match container.borrow().actual_item() {
            Item::Widget(widget) => {
                if widget.widget.widget_type != widget_type {
                    panic!(
                        "Active widget must be {:?} not {:?}.",
                        widget_type, widget.widget.widget_type
                    )
                }
            }
            Item::Container(_) => panic!("Actual item must be widget not container."),
        }
    }

    #[test]
    fn test_selecting_widget() {
        let c = create_testing_container();
        let check = |widget_type| match &Container::select_widget(c.clone(), widget_type) {
            Ok(c) => {
                check_active(c, widget_type);
                Ok(())
            }
            Err(_) => Err("Widget is not in container"),
        };

        check(Input).unwrap();
        check(List).unwrap();
        check(Done).unwrap();
        check(Project).unwrap();
        assert!(
            check(Context).is_err(),
            "Widget with type Context is not in container."
        );
    }

    #[test]
    fn test_next_item() -> Result<(), ErrorToDo> {
        let c = create_testing_container();

        // Test next widget in child container.
        let actual = Container::select_widget(c.clone(), List)?;
        let next = Container::next_item(actual.clone()).unwrap();
        check_active(&next, Done);

        // Test next widget in same container.
        let actual = Container::select_widget(c.clone(), Done)?;
        let next = Container::next_item(actual.clone()).unwrap();
        check_active(&next, Project);

        // Test next in container have not default value
        let actual = Container::select_widget(c.clone(), List)?;
        let next = Container::next_item(actual.clone()).unwrap();
        check_active(&next, Project);

        // Test return value if there is no next item
        assert!(Container::next_item(actual.clone()).is_none());
        assert!(Container::next_item(actual.clone()).is_none());
        assert!(Container::next_item(actual.clone()).is_none());
        assert_eq!(actual.borrow().act_index, 1);
        check_active(&next, Project);

        Ok(())
    }

    #[test]
    fn test_previous_item() -> Result<(), ErrorToDo> {
        let c = create_testing_container();

        // Test previous widget in same container.
        let actual = Container::select_widget(c.clone(), Project)?;
        let prev = Container::previous_item(actual.clone()).unwrap();
        check_active(&prev, Done);

        // Test return value if there is no previous item
        assert!(Container::previous_item(prev.clone()).is_none());
        assert!(Container::previous_item(prev.clone()).is_none());
        assert!(Container::previous_item(prev.clone()).is_none());
        assert_eq!(prev.borrow().act_index, 0);
        check_active(&prev, Done);

        Ok(())
    }
}
