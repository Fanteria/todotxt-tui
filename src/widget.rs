use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame, 
};

#[allow(dead_code)]
pub enum WidgetType {
    Input,
    List,
    Done,
    Categories,
}

#[allow(dead_code)]
pub struct Widget {
    pub widget_type: WidgetType,
    pub chunk: Rect,
}

#[allow(dead_code)]
impl Widget {
    pub fn new(widget_type: WidgetType, chunk: Rect) -> Widget {
        Widget { widget_type, chunk }
    }

    pub fn draw<B>(&self, f: &mut Frame<B>)
    where
        B: Backend,
    {
        match self.widget_type {
            WidgetType::Input => {
                draw_input(f, &self.chunk);
            }
            WidgetType::List => {
                draw_list(f, &self.chunk);
            }
            WidgetType::Done => {
                draw_done(f, &self.chunk);
            }
            _ => {}
        }
    }
}

fn draw_input<B>(f: &mut Frame<B>, chunk: &Rect)
where
    B: Backend,
{
    f.render_widget(
        Paragraph::new("Some text").block(Block::default().title("Firs").borders(Borders::ALL)),
        *chunk,
    );
}

fn draw_list<B>(f: &mut Frame<B>, chunk: &Rect)
where
    B: Backend,
{
    f.render_widget(
        Block::default().title("ToDo list").borders(Borders::ALL),
        *chunk,
    );
}

fn draw_done<B>(f: &mut Frame<B>, chunk: &Rect)
where
    B: Backend,
{
    f.render_widget(Block::default().title("Done").borders(Borders::ALL), *chunk);
}
