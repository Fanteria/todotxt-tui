mod layout;
mod widget;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    ExecutableCommand,
};
use layout::Layout;
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::Rect,
    // widgets::{Block, Borders, Paragraph, Widget},
    Terminal,
};

// fn render<B>(
//     terminal: &mut Terminal<B>,
//     chunks: &Vec<Rect>,
//     body_chunks: &Vec<Rect>,
// ) -> Result<(), io::Error>
// where
//     B: Backend,
// {
//     terminal.draw(|f| {
//         f.render_widget(
//             Paragraph::new("Some text").block(Block::default().title("Input").borders(Borders::ALL)),
//             chunks[0],
//         );
//         f.render_widget(
//             Block::default().title("Firs").borders(Borders::ALL),
//             body_chunks[0],
//         );
//         f.render_widget(
//             Block::default().title("Second").borders(Borders::ALL),
//             body_chunks[1],
//         );
//     })?;
//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    draw_ui().await?;

    Ok(())
}

async fn draw_ui() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let mut backend = CrosstermBackend::new(io::stdout());
    backend.execute(SetTitle("Title"))?; // TODO set window title

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut layout = Layout::new(terminal.size()?);
    terminal.draw(|f| {
        layout.render(f);
    })?;

    // let layout = Layout::default()
    //     .direction(Direction::Vertical)
    //     .constraints([Constraint::Length(3), Constraint::Percentage(30)].as_ref());
    //
    // let chunks = layout.split(terminal.size()?);
    // let chunks2 = layout.split(terminal.size()?);

    // let body_chunks = Layout::default()
    //     .direction(Direction::Horizontal)
    //     .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
    //     .split(chunks[1]);
    // render(&mut terminal, &chunks, &body_chunks)?;

    // main loop
    loop {
        // let mut app = app.lock().await;
        match event::read()? {
            Event::Resize(width, height) => {
                layout.update_chunk(Rect::new(0, 0, width, height));
            }
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => {
                    break;
                }
                _ => {}
            },
            _ => {}
        }

        terminal.draw(|f| {
            layout.render(f);
        })?;

        // render(&mut terminal, &chunks, &body_chunks)?;
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
