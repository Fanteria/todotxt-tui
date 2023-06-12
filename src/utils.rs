use crate::CONFIG;
use tui::{
    style::Style,
    widgets::{Block, BorderType, Borders},
};

macro_rules! some_or_return {
    ($message:expr) => {
        match $message {
            Some(s) => s,
            None => return,
        }
    };
}

pub(crate) use some_or_return;

pub fn get_block(title: &str, active: bool) -> Block {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title.clone())
        .border_type(BorderType::Rounded);
    if active {
        block = block.border_style(Style::default().fg(CONFIG.active_color));
    }
    block
}

