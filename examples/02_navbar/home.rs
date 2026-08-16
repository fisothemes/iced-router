//! Home screen.
//!
//! It owns its state, runs a task that sends a message back to itself,
//! and asks the router to go to Settings.

use crate::{Id, Shared};
use iced::widget::{button, column, text};
use iced::{Element, Task};
use iced_router::{Action, Screen};

#[derive(Debug, Clone)]
pub enum Message {
    GreetPressed,
    Greeted,
    SettingsPressed,
}

#[derive(Default)]
pub struct Home {
    busy: bool,
}

impl Screen<Id, Shared> for Home {
    type Message = Message;

    fn title(&self, _shared: &Shared) -> Option<String> {
        Some(String::from("Home"))
    }

    fn update(&mut self, shared: &mut Shared, message: Message) -> Action<Id, Message> {
        match message {
            Message::GreetPressed => {
                self.busy = true;

                // The task returns `Message::Greeted` to this screen only.
                // The router tags it with Home's handle, so it comes back here
                // even if the user already switched to another tab.
                Action::Run(Task::perform(
                    async {
                        println!("Doing a long-running greeting...");
                    },
                    |_| Message::Greeted,
                ))
            }
            Message::Greeted => {
                self.busy = false;

                // Shared state is how screens talk. No message crosses a boundary.
                shared.greeted += 1;

                Action::None
            }
            // Ask the router to navigate to Settings.
            Message::SettingsPressed => Action::GoTo(Id::Settings),
        }
    }

    fn view(&self, shared: &Shared) -> Element<'_, Message> {
        column![
            text(format!("Hello, {}", shared.username)).size(24),
            text(format!("Greeted {} times", shared.greeted)),
            button(if self.busy { "Greeting..." } else { "Greet" })
                .on_press_maybe((!self.busy).then_some(Message::GreetPressed)),
            button("Open settings").on_press(Message::SettingsPressed),
        ]
        .height(iced::Fill)
        .width(iced::Fill)
        .align_x(iced::Center)
        .spacing(12)
        .into()
    }
}
