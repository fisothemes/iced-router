//! About screen.
//!
//! Demonstrates `Action::Back`.

use iced::Element;
use iced::widget::{button, column, container, text};
use iced_router::{Action, Screen};

use crate::Id;

#[derive(Debug, Clone)]
pub enum Message {
    BackPressed,
}

pub struct About;

impl Screen<Id> for About {
    type Message = Message;

    fn update(&mut self, _shared: &mut (), message: Message) -> Action<Id, Message> {
        match message {
            // Go back to the previous screen.
            Message::BackPressed => Action::Back,
        }
    }

    fn view(&self, _shared: &()) -> Element<'_, Message> {
        let page = column![
            text("About").size(24),
            text("A router for iced applications."),
            button("Back").on_press(Message::BackPressed),
        ]
        .align_x(iced::Center)
        .spacing(12);

        container(page)
            .width(iced::Fill)
            .height(iced::Fill)
            .center_x(iced::Fill)
            .center_y(iced::Fill)
            .into()
    }
}
