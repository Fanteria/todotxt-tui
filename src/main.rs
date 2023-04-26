use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Widget},
    Terminal,
};

fn render<B>(terminal: &mut Terminal<B>, chunks: &Vec<Rect>, body_chunks: &Vec<Rect>) -> Result<(), io::Error>
where
    B: Backend,
{
    terminal.draw(|f| {
        f.render_widget(
            Paragraph::new("Some text").block(Block::default().title("Firs").borders(Borders::ALL)),
            chunks[0],
        );
        f.render_widget(
            Block::default().title("Firs").borders(Borders::ALL),
            body_chunks[0],
        );
        f.render_widget(
            Block::default().title("Second").borders(Borders::ALL),
            body_chunks[1],
        );
    })?;
    Ok(())
}

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // setup layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(terminal.size()?);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    render(&mut terminal, &chunks, &body_chunks)?;
    loop {
        if let Ok(Event::Key(event)) = event::read() {
            match event.code {
                KeyCode::Char('q') => {
                    break;
                }
                _ => {}
            }
            render(&mut terminal, &chunks, &body_chunks)?;
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
