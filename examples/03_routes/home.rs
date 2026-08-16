//! Home screen.
//!
//! It turns a typed query into an id and navigates.
//! The query stays in the id and nothing is stored in shared state.

use crate::Id;
use iced::widget::{button, column, text, text_input};
use iced::{Element, Length};
use iced_router::{Action, Screen};

#[derive(Debug, Clone)]
pub enum Message {
    DraftChanged(String),
    Submitted,
}

#[derive(Default)]
pub struct Home {
    draft: String,
}

impl Screen<Id> for Home {
    type Message = Message;

    fn title(&self, _shared: &()) -> Option<String> {
        Some(String::from("Search"))
    }

    fn update(&mut self, _shared: &mut (), message: Message) -> Action<Id, Message> {
        match message {
            Message::DraftChanged(draft) => {
                self.draft = draft;

                Action::None
            }
            // The query goes in the id. There is no shared field to write and
            // nothing to clear afterwards.
            Message::Submitted => match self.draft.trim() {
                "" => Action::None,
                query => Action::GoTo(Id::Search {
                    query: query.to_owned(),
                    page: 1,
                }),
            },
        }
    }

    fn view<'a>(&'a self, _shared: &'a ()) -> Element<'a, Message> {
        column![
            text("Search").size(24),
            text_input(r#"Try "a""#, &self.draft)
                .on_input(Message::DraftChanged)
                .on_submit(Message::Submitted),
            button("Search").on_press(Message::Submitted),
        ]
        .width(Length::Fill)
        .spacing(12)
        .padding(24)
        .into()
    }
}
