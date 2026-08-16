//! User screen.
//!
//! Built by a route, from the id in `Id::User`.
//!
//! Two different user ids are two different screens. Opening user 3 and then user 7
//! leaves both in the history. Back returns to user 3 with its own state.

use crate::{Id, person};
use iced::widget::{button, column, text};
use iced::{Element, Length, Task};
use iced_router::{Action, Screen};

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Option<String>),
    BackPressed,
}

pub struct User {
    id: u64,
    name: Option<String>,
    loaded: bool,
}

impl User {
    /// The route calls this with the id read from `Id::User`.
    pub fn new(id: u64) -> Self {
        Self {
            id,
            name: None,
            loaded: false,
        }
    }
}

impl Screen<Id> for User {
    type Message = Message;

    /// The screen knows which user it is, so it starts loading.
    fn on_enter(&mut self, _shared: &mut ()) -> Task<Message> {
        if self.loaded {
            return Task::none();
        }

        Task::perform(fetch(self.id), Message::Loaded)
    }

    fn title(&self, _shared: &()) -> Option<String> {
        self.name.clone()
    }

    fn update(&mut self, _shared: &mut (), message: Message) -> Action<Id, Message> {
        match message {
            Message::Loaded(name) => {
                self.name = name;
                self.loaded = true;

                Action::None
            }
            // Back does not name a screen, so this screen doesn't need to know
            // which search sent the user here.
            Message::BackPressed => Action::Back,
        }
    }

    fn view<'a>(&'a self, _shared: &'a ()) -> Element<'a, Message> {
        let body = match (&self.name, self.loaded) {
            (Some(name), _) => text(name.as_str()).size(24),
            (None, false) => text("Loading..."),
            (None, true) => text("No such user."),
        };

        column![
            button("Back").on_press(Message::BackPressed),
            body,
            text(format!("id {}", self.id)),
        ]
        .width(Length::Fill)
        .spacing(12)
        .padding(24)
        .into()
    }
}

/// Stands in for a network call.
async fn fetch(id: u64) -> Option<String> {
    person(id).map(|name| name.to_owned())
}
