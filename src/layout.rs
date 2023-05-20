mod container;
mod widget;
use self::container::InitItem;
use crate::error::ErrorToDo;
use container::Container;
use std::cell::RefCell;
use std::rc::Rc;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Direction::Horizontal, Direction::Vertical, Rect},
    Frame,
};
use widget::{Widget, WidgetType};

type RcCon = Rc<RefCell<Container>>;

pub struct Layout {
    root: Rc<RefCell<Container>>,
    actual: Rc<RefCell<Container>>,
}

impl Layout {
    pub fn new(chunk: Rect) -> Layout {
        let input_widget = Widget::new(WidgetType::Input, "Input");
        let list_widget = Widget::new(WidgetType::List, "List");
        let done_widget = Widget::new(WidgetType::Done, "Done");
        let categories_widget = Widget::new(WidgetType::Categories, "Categories");

        let root = Container::new(
            vec![
                InitItem::Widget(input_widget),
                InitItem::Container(Container::new(
                    vec![
                        InitItem::Widget(list_widget),
                        InitItem::Container(Container::new(
                            vec![
                                InitItem::Widget(done_widget),
                                InitItem::Widget(categories_widget),
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
        let actual = Container::select_widget(&root, &WidgetType::List).unwrap(); // TODO
        root.borrow_mut().update_chunks(chunk);

        Layout { root, actual }
    }

    pub fn move_focus(
        container: RcCon,
        direction: Direction,
        f: fn(RcCon) -> Option<RcCon>,
    ) -> RcCon {
        if container.borrow().direction == direction {
            return match f(container.clone()) {
                Some(actual) => actual,
                None => {
                    {
                        container.borrow_mut().active = false;
                    }
                    match &container.borrow().parent {
                        Some(parent) => {
                            // container.borrow_mut().active = false;
                            Layout::move_focus(parent.clone(), direction, f)
                        }
                        None => container.clone(),
                    }
                    //Rc::clone(&container)}, //TODO wrong
                }
            };
        }
        let mut c = container.borrow_mut();
        return match &c.parent {
            Some(parent) => {
                let ret = Layout::move_focus(parent.clone(), direction, f);
                c.active = false;
                ret
            }
            None => container.clone(),
        };
    }

    pub fn left(&mut self) {
        self.actual = Layout::move_focus(
            Rc::clone(&self.actual),
            Horizontal,
            Container::previous_item,
        );
    }

    pub fn right(&mut self) {
        self.actual = Layout::move_focus(Rc::clone(&self.actual), Horizontal, Container::next_item);
    }

    pub fn up(&mut self) {
        self.actual =
            Layout::move_focus(Rc::clone(&self.actual), Vertical, Container::previous_item);
    }

    pub fn down(&mut self) {
        self.actual = Layout::move_focus(Rc::clone(&self.actual), Vertical, Container::next_item);
    }

    #[allow(dead_code)]
    pub fn select_widget(&mut self, widget_type: &WidgetType) -> Result<(), ErrorToDo> {
        self.actual = Container::select_widget(&self.root, widget_type)?;
        Ok(())
    }

    pub fn update_chunks(&mut self, chunk: Rect) {
        self.root.borrow_mut().update_chunks(chunk);
    }

    pub fn render<B>(&self, f: &mut Frame<B>)
    where
        B: Backend,
    {
        self.root.as_ref().borrow().render_recursive(f);
    }
}
