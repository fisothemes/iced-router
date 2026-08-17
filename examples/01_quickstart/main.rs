//! Example 01: Quick start.
//! Run with: `cargo run --example 01_quickstart`
//!
//! This is the fastest way to start. Use `iced_router::application` when the
//! router draws the whole window. If you need a navbar, sidebar, or anything
//! else around the router, see Example 02 instead.
//!
//! There is no `App` struct here. `boot` builds the router and hands it to
//! `iced_router::application`, which does the rest.

use iced::Task;
use iced::window::{Position as WindowPosition, Settings as WindowSettings};
use iced_router::{Navigation, Router};

mod about;
mod home;

const WINDOW_SIZE: iced::Size = iced::Size::new(300.0, 240.0);

fn main() -> iced::Result {
    iced_router::application(boot)
        .title("Quick start")
        .window(WindowSettings {
            min_size: WINDOW_SIZE.into(),
            size: WINDOW_SIZE,
            position: WindowPosition::Centered,
            ..Default::default()
        })
        .run()
}

fn boot() -> (Router<Id>, (), Task<iced_router::Message<Id>>) {
    // Build the router. Screens live for the whole app.
    // The router owns no shared state (S = ()), so we pass ().
    Router::builder(Id::Home)
        .navigation(Navigation::Stack)
        .screen(Id::Home, home::Home)
        .screen(Id::About, about::About)
        .build(())
}

/// Screen identifiers. This is the only way one screen names another.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Id {
    Home,
    About,
}
