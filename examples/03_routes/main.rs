//! Example 03: Routes with arguments.
//! Run with: `cargo run --example 03_routes`
//!
//! This example demonstrates screens that take arguments. `Id::Search` carries
//! a query and page number. `Id::User` carries a user id. The route builds the
//! screen from those values when the router first goes there.
//!
//! Nothing is stored in shared state. The shared state is `()`.
//!
//! The router uses `Navigation::Stack`, so a new search is a new screen.
//! Search twice, press Back, and the first search is still there with its results.
//! Press Back again and that screen is dropped..
//!
//! ## Note
//!
//! `.screen(...)` and `.route(...)` are different. `Home` is registered
//! with `.screen(...)`, so it is built once and lives for as long as the
//! application. Search and User are built by a route, so each one lives only
//! while its id is in the history.

use iced::Task;
use iced::window::{Position as WindowPosition, Settings as WindowSettings};
use iced_router::{Message, Navigation, Router, Screen};

mod home;
mod search;
mod user;

const WINDOW_SIZE: iced::Size = iced::Size::new(420.0, 480.0);

fn main() -> iced::Result {
    iced_router::application(boot)
        .title("Routes")
        .window(WindowSettings {
            min_size: WINDOW_SIZE.into(),
            size: WINDOW_SIZE,
            position: WindowPosition::Centered,
            ..Default::default()
        })
        .run()
}

fn boot() -> (Router<Id>, Task<Message<Id>>) {
    Router::builder(Id::Home, ())
        .navigation(Navigation::Stack)
        // Built once, kept for the lifetime of the application.
        .screen(Id::Home, home::Home::default())
        // Built on request, from the values in the id. Return `None` or a fallback screen
        // for an id this function does not recognise.
        .route(|id| match id {
            Id::Search { query, page } => Some(search::Search::new(query, *page).boxed()),
            Id::User { id } => Some(user::User::new(*id).boxed()),
            _ => None,
        })
        .build()
}

/// Screen identifiers. An id carries everything its screen needs to be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Id {
    Home,
    Search { query: String, page: u32 },
    User { id: u64 },
}

/// The data both `search` and `user` read. Data is not a screen, so sharing a
/// module here does not couple the two screens together.
pub const PEOPLE: [(u64, &str); 8] = [
    (1, "Ada Lovelace"),
    (2, "Alan Turing"),
    (3, "Grace Hopper"),
    (4, "Barbara Liskov"),
    (5, "Edsger Dijkstra"),
    (6, "Katherine Johnson"),
    (7, "Donald Knuth"),
    (8, "Margaret Hamilton"),
];

pub fn person(id: u64) -> Option<&'static str> {
    PEOPLE
        .iter()
        .find(|(other, _)| *other == id)
        .map(|(_, name)| *name)
}
