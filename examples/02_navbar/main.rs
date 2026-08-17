//! Example 02: A navbar outside the router.
//! Run with: `cargo run --example 02_navbar`
//!
//! This example demonstrates a navbar that sits outside the router.
//!
//! The navbar highlights the active screen.
//! The router uses `Navigation::Switch` so clicking a link replaces the
//! current screen, never pushes it into history.
//!
//! Screens don't know the navbar exists. They only know `Id` and `Shared`.

use iced::widget::{button, column, container, row};
use iced::window::{Position as WindowPosition, Settings as WindowSettings};
use iced::{Element, Task};
use iced_router::{Navigation, Router};

mod home;
mod settings;

const WINDOW_SIZE: iced::Size = iced::Size::new(420.0, 360.0);

fn main() -> iced::Result {
    iced::application(boot, App::update, App::view)
        .title(App::title)
        .window(WindowSettings {
            min_size: WINDOW_SIZE.into(),
            size: WINDOW_SIZE,
            position: WindowPosition::Centered,
            ..Default::default()
        })
        .run()
}

fn boot() -> (App, Task<Message>) {
    // 1. Build the router. Screens live for the whole app lifetime.
    //    If a screen needs a fresh state each visit, reset it in `Screen::on_enter`.
    let (router, shared, task) = Router::builder(Id::Home)
        .navigation(Navigation::Switch)
        .screen(Id::Home, home::Home::default())
        .screen(Id::Settings, settings::Settings::default())
        .build(Shared::default());

    (App { router, shared }, task)
}

struct App {
    router: Router<Id, Shared>,
    shared: Shared,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        // 2. Pass every message straight to the router.
        self.router.update(&mut self.shared, message)
    }

    fn view(&self) -> Element<'_, Message> {
        // 3. Draw the navbar above the current screen.
        column![
            navbar(self.router.current()),
            container(self.router.view(&self.shared)).padding(24),
        ]
        .into()
    }

    fn title(&self) -> String {
        // 4. Each screen sets its own title via `Screen::title`.
        self.router
            .title(&self.shared)
            .unwrap_or_else(|| String::from("Navbar Example"))
    }
}

/// The application's message type. It's also the router's message type.
type Message = iced_router::Message<Id>;

/// Screen identifiers. These are used to transition between screens.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Id {
    Home,
    Settings,
}

impl Id {
    const ALL: [Id; 2] = [Id::Home, Id::Settings];

    fn label(self) -> &'static str {
        match self {
            Id::Home => "Home",
            Id::Settings => "Settings",
        }
    }
}

/// State shared between screens.
#[derive(Debug)]
pub struct Shared {
    pub username: String,
    pub greeted: u32,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            username: String::from("guest"),
            greeted: 0,
        }
    }
}

/// The navbar. Reads the current id to highlight the active link.
/// Produces `Message::GoTo` directly.
fn navbar(current: &Id) -> Element<'_, Message> {
    let links = Id::ALL.into_iter().fold(row![].spacing(8), |links, id| {
        let active = id == *current;

        links.push(
            button(id.label())
                .style(move |theme, status| {
                    if active {
                        button::primary(theme, status)
                    } else {
                        button::text(theme, status)
                    }
                })
                .on_press(Message::GoTo(id)),
        )
    });

    container(links).padding(12).into()
}
