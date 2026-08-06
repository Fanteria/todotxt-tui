use anyhow::{Context, Result};
use crossterm::{clipboard::CopyToClipboard, execute};
use std::io;

/// Copies the given text into the system clipboard.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    execute!(io::stdout(), CopyToClipboard::to_clipboard_from(text))
        .context("Failed to write the clipboard escape sequence")
}
