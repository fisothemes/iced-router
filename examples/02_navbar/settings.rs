//! Settings screen.
//!
//! Demonstrates `Screen::on_enter`.
//!
//! Screens are created once and kept alive. If a screen needs a fresh state
//! on every visit, set it up in `on_enter`.
//!
//! This screen writes the username that Home reads. They never exchange a message.

use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Task};
use iced_router::{Action, Screen};

use crate::{Id, Shared};

#[derive(Debug, Clone)]
pub enum Message {
    UsernameChanged(String),
    Save,
    Done,
}

#[derive(Default)]
pub struct Settings {
    draft: String,
}

impl Screen<Id, Shared> for Settings {
    type Message = Message;

    // Runs every time the router shows this screen.
    // Reload the saved name into the draft.
    fn on_enter(&mut self, shared: &mut Shared) -> Task<Message> {
        self.draft = shared.username.clone();
        Task::none()
    }

    fn title(&self, _shared: &Shared) -> Option<String> {
        Some(String::from("Settings"))
    }

    fn update(&mut self, shared: &mut Shared, message: Message) -> Action<Id, Message> {
        match message {
            // Draft stays local until saved.
            Message::UsernameChanged(name) => {
                self.draft = name;
                Action::None
            }
            // Save copies the draft into shared state.
            // Home picks it up when it redraws.
            Message::Save => {
                shared.username = self.draft.clone();
                Action::None
            }
            Message::Done => Action::GoTo(Id::Home),
        }
    }

    fn view(&self, shared: &Shared) -> Element<'_, Message> {
        let dirty = self.draft != shared.username;

        column![
            text("Settings").size(24),
            row![
                text_input("Username", &self.draft).on_input(Message::UsernameChanged),
                button("Save").on_press_maybe(dirty.then_some(Message::Save)),
            ]
            .spacing(8),
            button("Done").on_press(Message::Done),
        ]
        .spacing(12)
        .into()
    }
}
