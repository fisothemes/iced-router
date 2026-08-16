//! Home screen.
//!
//! It only navigates to About.

use iced::Element;
use iced::widget::{button, column, container, text};
use iced_router::{Action, Screen};

use crate::Id;

#[derive(Debug, Clone)]
pub enum Message {
    AboutPressed,
}

pub struct Home;

impl Screen<Id> for Home {
    type Message = Message;

    fn update(&mut self, _shared: &mut (), message: Message) -> Action<Id, Message> {
        match message {
            // Ask the router to go to About.
            Message::AboutPressed => Action::GoTo(Id::About),
        }
    }

    fn view(&self, _shared: &()) -> Element<'_, Message> {
        let page = column![
            text("Home").size(24),
            button("About").on_press(Message::AboutPressed),
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
