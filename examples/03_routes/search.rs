//! Search screen.
//!
//! Built by a route, from the query and page in the id.
//!
//! `new` sets up the state. `on_enter` starts the work, because a constructor
//! cannot return a `Task`. `on_enter` runs on every arrival, so it checks if
//! results are already loaded and does nothing when the user came back via Back.

use crate::{Id, PEOPLE};
use iced::widget::{button, column, row, text};
use iced::{Element, Length, Task};
use iced_router::{Action, Screen};

const PER_PAGE: usize = 3;

#[derive(Debug, Clone)]
pub struct Hit {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Vec<Hit>),
    NextPage,
    HitPressed(u64),
    BackPressed,
}

pub struct Search {
    query: String,
    page: u32,
    hits: Vec<Hit>,
    loaded: bool,
}

impl Search {
    /// The route calls this with the values it read from the id.
    pub fn new(query: &str, page: u32) -> Self {
        Self {
            query: query.to_owned(),
            page,
            hits: Vec::new(),
            loaded: false,
        }
    }
}

impl Screen<Id> for Search {
    type Message = Message;

    /// Runs on every arrival, including a Back from the User screen.
    /// If results are already loaded, do nothing.
    fn on_enter(&mut self, _shared: &mut ()) -> Task<Message> {
        if self.loaded {
            return Task::none();
        }

        Task::perform(find(self.query.clone(), self.page), Message::Loaded)
    }

    fn title(&self, _shared: &()) -> Option<String> {
        Some(format!("{} ({})", self.query, self.page))
    }

    fn update(&mut self, _shared: &mut (), message: Message) -> Action<Id, Message> {
        match message {
            Message::Loaded(hits) => {
                self.hits = hits;
                self.loaded = true;

                Action::None
            }
            // The next page is a different id, so it is a different screen.
            // Back returns to this one with its results still loaded.
            Message::NextPage => Action::GoTo(Id::Search {
                query: self.query.clone(),
                page: self.page + 1,
            }),
            Message::HitPressed(id) => Action::GoTo(Id::User { id }),
            Message::BackPressed => Action::Back,
        }
    }

    fn view<'a>(&'a self, _shared: &'a ()) -> Element<'a, Message> {
        let hits: Element<'_, Message> = if !self.loaded {
            text("Searching...").into()
        } else if self.hits.is_empty() {
            text("No results.").into()
        } else {
            self.hits
                .iter()
                .fold(column![].spacing(4), |list, hit| {
                    list.push(
                        button(text(&hit.name))
                            .width(Length::Fill)
                            .on_press(Message::HitPressed(hit.id)),
                    )
                })
                .into()
        };

        column![
            row![
                button("Back").on_press(Message::BackPressed),
                text(format!("\"{}\", page {}", self.query, self.page)),
            ]
            .spacing(8),
            hits,
            button("Next page")
                .on_press_maybe((self.hits.len() == PER_PAGE).then_some(Message::NextPage)),
        ]
        .width(Length::Fill)
        .spacing(12)
        .padding(24)
        .into()
    }
}

/// Stands in for a network call.
async fn find(query: String, page: u32) -> Vec<Hit> {
    let query = query.to_lowercase();
    let skip = (page.saturating_sub(1) as usize) * PER_PAGE;

    PEOPLE
        .iter()
        .filter(|(_, name)| name.to_lowercase().contains(&query))
        .skip(skip)
        .take(PER_PAGE)
        .map(|(id, name)| Hit {
            id: *id,
            name: (*name).to_owned(),
        })
        .collect()
}
