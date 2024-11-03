use tui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

use crate::config::WidgetBorderType;

pub struct Popup {
    border_type: WidgetBorderType,
    message: Option<String>,
}

impl Popup {
    pub fn new(border_type: WidgetBorderType) -> Self {
        Self {
            border_type,
            message: None,
        }
    }

    fn center_popup_area(area: Rect, percent_width: u16, percent_height: u16) -> Rect {
        let vertical =
            Layout::vertical([Constraint::Percentage(percent_height)]).flex(Flex::Center);
        let horizontal =
            Layout::horizontal([Constraint::Percentage(percent_width)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    pub fn render_popup(&mut self, frame: &mut Frame) {
        if let Some(message) = self.message.take() {
            let area = Self::center_popup_area(frame.area(), 50, 25);
            frame.render_widget(Clear, area);
            frame.render_widget(
                Paragraph::new(message).block(
                    Block::bordered()
                        .border_type(self.border_type.into())
                        .title("Error"),
                ),
                area,
            );
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.message = Some(message);
    }
}
