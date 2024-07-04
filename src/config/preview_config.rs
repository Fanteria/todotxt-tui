use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Parser, Debug, PartialEq, Eq, Clone)]
pub struct PreviewConfig {

    /// Preview format (uses placeholders).
    #[arg(short, long, default_value_t = default_preview_foramt(), hide_default_value = true)]
    #[serde(default = "default_preview_foramt")]
    pub preview_format: String,

    /// Wrap long lines in the preview.
    #[arg(short, long, default_value_t = default_wrap_preview())]
    #[serde(default = "default_wrap_preview")]
    pub wrap_preview: bool,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            preview_format: default_preview_foramt(),
            wrap_preview: default_wrap_preview(),
        }
    }
}

fn default_preview_foramt() -> String {
    String::from(
        "Pending: $pending Done: $done
Subject: $subject
Priority: $priority
Create date: $create_date
Link: $link",
    )
}

fn default_wrap_preview() -> bool {
    true
}
