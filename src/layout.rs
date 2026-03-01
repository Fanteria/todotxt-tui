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
use std::str::FromStr;
use tui::{
    layout::{Constraint, Direction, Rect},
    Frame,
};
use widget::{new_widget, WidgetType};

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
    pub fn from_str(template: &str, todo: &ToDo, config: &Config) -> Result<Self> {
        fn read_block(
            conts: &mut Vec<Container>,
            parent: Option<usize>,
            block: Pair<'_, Rule>,
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
                        conts[index].add_widget(new_widget(
                            WidgetType::from_str(widget_blocks[0].as_str())?,
                            config,
                        )?);
                    }
                    Rule::block => {
                        constrains.push(act_constrain.take().unwrap_or(Constraint::Percentage(50)));
                        let next_item = read_block(conts, Some(index), inner, config)?;
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
        read_block(&mut containers, None, outer_block, config)?;

        let mut layout = Self { containers, act: 0 };
        Container::actualize_layout(&mut layout);
        layout.act_mut().actual_mut().unwrap().focus(todo);
        Ok(layout)
    }

    fn act(&self) -> &Container {
        &self.containers[self.act]
    }

    fn act_mut(&mut self) -> &mut Container {
        &mut self.containers[self.act]
    }

    fn walk_in_container(&mut self, f: &impl Fn(&mut Container) -> bool, todo: &ToDo) -> bool {
        if f(self.act_mut()) {
            Container::actualize_layout(self);
            match self.act_mut().actual_mut() {
                Some(widget) => widget.focus(todo) || self.walk_in_container(f, todo),
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
    fn change_focus(
        &mut self,
        direction: &Direction,
        f: &impl Fn(&mut Container) -> bool,
        todo: &ToDo,
    ) -> bool {
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
                Some(widget) => widget.focus(todo) || self.walk_in_container(f, todo),
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
                    if self.change_focus(direction, f, todo) {
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
    fn move_focus(
        &mut self,
        direction: Direction,
        function: fn(&mut Container) -> bool,
        todo: &ToDo,
    ) -> bool {
        let ret = self.change_focus(&direction, &function, todo);
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
    pub fn left(&mut self, todo: &ToDo) -> bool {
        self.move_focus(Direction::Horizontal, Container::previous_item, todo)
    }

    /// Move the focus to the right.
    pub fn right(&mut self, todo: &ToDo) -> bool {
        self.move_focus(Direction::Horizontal, Container::next_item, todo)
    }

    /// Move the focus upwards.
    pub fn up(&mut self, todo: &ToDo) -> bool {
        self.move_focus(Direction::Vertical, Container::previous_item, todo)
    }

    /// Move the focus downwards.
    pub fn down(&mut self, todo: &ToDo) -> bool {
        self.move_focus(Direction::Vertical, Container::next_item, todo)
    }

    /// Handle a key event.
    ///
    /// This method is used to handle key events within the layout. It passes the key event to the
    /// currently focused widget or container for processing.
    ///
    /// # Parameters
    ///
    /// - `event`: A reference to the `KeyEvent` to be handled.
    pub fn handle_key(&mut self, event: &KeyEvent, todo: &mut ToDo) -> bool {
        self.act_mut()
            .actual_mut()
            .expect("Actual is not widget")
            .handle_key(event, todo)
    }

    pub fn get_active_widget(&self) -> WidgetType {
        self.act().get_active_type().expect("Actual is not widget")
    }

    fn find_widget(
        &mut self,
        find_functor: impl Fn(&&mut Box<dyn State>) -> bool,
        process_widget: impl Fn(&mut dyn State),
        todo: &ToDo,
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
                process_widget(widget.as_mut());
                if self.act == layout_index && cont_act_index == cont_index {
                    None
                } else if widget.focus(todo) {
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

    pub fn click(&mut self, column: u16, row: u16, todo: &ToDo) {
        log::debug!("Click on column {column}, row {row}");
        self.find_widget(
            |w| {
                let chunk = &w.get_base().chunk;
                let x = chunk.x < column && column < chunk.x + chunk.width;
                let y = chunk.y < row && row < chunk.y + chunk.height;
                x && y
            },
            |w| w.click(column.into(), row.into(), todo),
            todo,
        );
    }

    pub fn select_widget(&mut self, widget_type: WidgetType, todo: &ToDo) {
        log::debug!("Select widget {widget_type}");
        self.find_widget(|w| w.widget_type() == widget_type, |_| {}, todo);
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
    fn render(&self, f: &mut Frame, todo: &ToDo) {
        self.containers[0].render(f, &self.containers, todo);
    }

    fn unfocus(&mut self) {
        match self.act_mut().actual_mut() {
            Some(w) => w.unfocus(),
            None => panic!("Actual to unfocus is not a widget"),
        }
    }

    fn focus(&mut self, todo: &ToDo) -> bool {
        match self.act_mut().actual_mut() {
            Some(w) => w.focus(todo),
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
        Layout::from_str(mock_layout, &ToDo::default(), &Config::default()).unwrap()
    }

    #[test]
    fn test_basic_movement() -> Result<()> {
        let mut l = mock_layout();
        assert_eq!(l.get_active_widget(), WidgetType::List);

        assert!(l.right(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::Done);
        assert!(l.left(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::List);
        assert!(l.right(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::Done);
        assert!(!l.right(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::Done);
        assert!(l.down(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::Context);
        assert!(l.right(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::Project);
        assert!(!l.down(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::Project);
        assert!(l.left(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::Context);
        assert!(l.left(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::List);
        assert!(l.right(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::Context);
        assert!(l.left(&ToDo::default()));
        assert_eq!(l.get_active_widget(), WidgetType::List);
        assert!(!l.up(&ToDo::default()));
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

        let mut layout = Layout::from_str(str_layout, &ToDo::default(), &Config::default())?;
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

    /// Render tests helper
    fn assert_rendered(template: &str, width: u16, height: u16, tasks: &[&str], expected: &[&str]) {
        let mut todo = ToDo::default();
        for task in tasks {
            todo.new_task(task).unwrap();
        }
        let config = Config::default();
        let mut layout = Layout::from_str(template, &todo, &config).unwrap();
        layout.update_chunk(Rect::new(0, 0, width, height));

        let backend = tui::backend::TestBackend::new(width, height);
        let mut terminal = tui::Terminal::new(backend).unwrap();
        terminal.draw(|f| layout.render(f, &todo)).unwrap();
        let buf = terminal.backend().buffer();
        // Extract text-only lines from a terminal buffer (ignoring styles).
        let actual: Vec<String> = (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf.cell((x, y)).unwrap().symbol().to_string())
                    .collect()
            })
            .collect();
        let expected: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(actual, expected, "\nactual:\n{}", actual.join("\n"));
    }

    #[test]
    fn render_single_list() {
        assert_rendered(
            "[List]",
            20,
            5,
            &[],
            &[
                "╭list──────────────╮",
                "│                  │",
                "╰──────────────────╯",
                "                    ",
                "                    ",
            ],
        );
    }

    #[test]
    fn render_single_done() {
        assert_rendered(
            "[Done]",
            20,
            5,
            &[],
            &[
                "╭done──────────────╮",
                "│                  │",
                "╰──────────────────╯",
                "                    ",
                "                    ",
            ],
        );
    }

    #[test]
    fn render_single_preview() {
        assert_rendered(
            "[Preview]",
            20,
            5,
            &[],
            &[
                "╭preview───────────╮",
                "│                  │",
                "╰──────────────────╯",
                "                    ",
                "                    ",
            ],
        );
    }

    #[test]
    fn render_two_widgets_vertical() {
        assert_rendered(
            "[List, Done]",
            20,
            8,
            &[],
            &[
                "╭list──────────────╮",
                "│                  │",
                "│                  │",
                "╰──────────────────╯",
                "╭done──────────────╮",
                "│                  │",
                "│                  │",
                "╰──────────────────╯",
            ],
        );
    }

    #[test]
    fn render_two_widgets_horizontal() {
        assert_rendered(
            "[Direction: Horizontal, List, Done]",
            20,
            5,
            &[],
            &[
                "╭list────╮╭done────╮",
                "│        ││        │",
                "│        ││        │",
                "│        ││        │",
                "╰────────╯╰────────╯",
            ],
        );
    }

    #[test]
    fn render_three_widgets_vertical() {
        assert_rendered(
            "[List, Done, Preview]",
            20,
            9,
            &[],
            &[
                "╭list──────────────╮",
                "│                  │",
                "╰──────────────────╯",
                "╭done──────────────╮",
                "│                  │",
                "╰──────────────────╯",
                "╭preview───────────╮",
                "│                  │",
                "╰──────────────────╯",
            ],
        );
    }

    #[test]
    fn render_three_widgets_horizontal() {
        assert_rendered(
            "[Direction: Horizontal, List, Done, Projects]",
            30,
            5,
            &[],
            &[
                "╭list────╮╭done────╮╭project─╮",
                "│        ││        ││        │",
                "│        ││        ││        │",
                "│        ││        ││        │",
                "╰────────╯╰────────╯╰────────╯",
            ],
        );
    }

    #[test]
    fn render_nested_horizontal_in_vertical() {
        let template = "[List, [Direction: Horizontal, Done, Projects]]";
        assert_rendered(
            template,
            20,
            8,
            &[],
            &[
                "╭list──────────────╮",
                "│                  │",
                "│                  │",
                "╰──────────────────╯",
                "╭done────╮╭project─╮",
                "│        ││        │",
                "│        ││        │",
                "╰────────╯╰────────╯",
            ],
        );
    }

    #[test]
    fn render_nested_vertical_in_horizontal() {
        let template = "[Direction: Horizontal, List, [Direction: Vertical, Done, Preview]]";
        assert_rendered(
            template,
            20,
            8,
            &[],
            &[
                "╭list────╮╭done────╮",
                "│        ││        │",
                "│        ││        │",
                "│        │╰────────╯",
                "│        │╭preview─╮",
                "│        ││        │",
                "│        ││        │",
                "╰────────╯╰────────╯",
            ],
        );
    }

    #[test]
    fn render_with_percentage_sizes() {
        let template = "[Direction: Horizontal, List: 30%, Done: 70%]";
        assert_rendered(
            template,
            20,
            5,
            &[],
            &[
                "╭list╮╭done────────╮",
                "│    ││            │",
                "│    ││            │",
                "│    ││            │",
                "╰────╯╰────────────╯",
            ],
        );
    }

    #[test]
    fn render_with_raw_sizes() {
        let template = "[Direction: Horizontal, List: 8, Done: 12]";
        assert_rendered(
            template,
            20,
            5,
            &[],
            &[
                "╭list──╮╭done──────╮",
                "│      ││          │",
                "│      ││          │",
                "│      ││          │",
                "╰──────╯╰──────────╯",
            ],
        );
    }

    #[test]
    fn render_all_category_widgets() {
        let template = "[Direction: Horizontal, Contexts, Projects, Hashtags]";
        assert_rendered(
            template,
            30,
            5,
            &[],
            &[
                "╭context─╮╭project─╮╭hashtag─╮",
                "│        ││        ││        │",
                "│        ││        ││        │",
                "│        ││        ││        │",
                "╰────────╯╰────────╯╰────────╯",
            ],
        );
    }

    #[test]
    fn render_list_with_tasks() {
        assert_rendered(
            "[List]",
            20,
            5,
            &["Alpha", "Beta"],
            &[
                "╭list──────────────╮",
                "│Alpha             │",
                "╰──────────────────╯",
                "                    ",
                "                    ",
            ],
        );
    }

    #[test]
    fn render_horizontal_list_and_done_with_tasks() {
        assert_rendered(
            "[Direction: Horizontal, List, Done]",
            30,
            5,
            &["Pending task", "x Done task"],
            &[
                "╭list─────────╮╭done─────────╮",
                "│Pending task ││Done task    │",
                "│             ││             │",
                "│             ││             │",
                "╰─────────────╯╰─────────────╯",
            ],
        );
    }

    #[test]
    fn render_categories_with_tasks() {
        assert_rendered(
            "[Direction: Horizontal, Projects, Contexts]",
            24,
            5,
            &["Buy milk +shopping @home", "Code +dev @work"],
            &[
                "╭project───╮╭context───╮",
                "│dev       ││home      │",
                "│shopping  ││work      │",
                "│          ││          │",
                "╰──────────╯╰──────────╯",
            ],
        );
    }

    #[test]
    fn render_deeply_nested() {
        assert_rendered(
            "[Direction: Horizontal, List, [Done, [Direction: Horizontal, Contexts, Projects]]]",
            30,
            8,
            &[],
            &[
                "╭list─────────╮╭done─────────╮",
                "│             ││             │",
                "│             ││             │",
                "│             │╰─────────────╯",
                "│             │╭contex╮╭proje╮",
                "│             ││      ││     │",
                "│             ││      ││     │",
                "╰─────────────╯╰──────╯╰─────╯",
            ],
        );
    }

    #[test]
    fn render_complex_layout() {
        assert_rendered(
            "[Direction: Horizontal, Size: 50%, [List, Preview], [Direction: Vertical, Done, [Contexts, Projects]]]",
            40,
            10,
            &[],
            &[
                "╭list──────────────╮╭done──────────────╮",
                "│                  ││                  │",
                "│                  ││                  │",
                "│                  ││                  │",
                "╰──────────────────╯╰──────────────────╯",
                "╭preview───────────╮╭context─╮╭project─╮",
                "│                  ││        ││        │",
                "│                  ││        ││        │",
                "│                  ││        ││        │",
                "╰──────────────────╯╰────────╯╰────────╯",
            ],
        );
    }
}
