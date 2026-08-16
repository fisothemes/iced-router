# Iced Router

Screen routing for [iced-rs](https://github.com/iced-rs/iced).

Each screen owns its own state, message and view. Screens never import each other.
An `update` returns an `Action` and router decides what to do with it.

```rust
use iced_router::{Action, Screen};
use iced::Element;

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
            Message::AboutPressed => Action::GoTo(Id::About),
        }
    }

    fn view(&self, _shared: &()) -> Element<'_, Message> {
        button("About").on_press(Message::AboutPressed).into()
    }
}
```

## Status

Not published to crates.io yet.

`iced-router` has its own version. The iced release it is built against is listed
here:

| iced-router | iced |
|-------------|------|
| 0.1         | 0.14 |

## Quick start

Add this to your `Cargo.toml`:

```toml
[dependencies]
iced = "0.14.0"
iced-router = { git = "https://github.com/fisothemes/iced-router", tag = "0.1.0-alpha.1" }
```

`iced_router::application` runs a router as a whole application.

```rust
fn main() -> iced::Result {
    iced_router::application(boot).title("Quick start").run()
}

fn boot() -> (Router<Id>, Task<Message<Id>>) {
    Router::builder(Id::Home, ())
        .navigation(Navigation::Stack)
        .screen(Id::Home, home::Home)
        .screen(Id::About, about::About)
        .build()
}
```

## Parts

- **`Screen`**: is one screen. It contains the `update`, `view`, and `subscription`
  methods, like an iced application, plus `on_enter`, `on_exit`, and `title`. How
  long a screen lives depends on how it was registered. See `.screen` and `.route`
  below.

- **`Action`**: is what `Screen::update` returns (`None`, `Run(Task)`, `GoTo`,
  `Back`, `Replace`, or `Reset`). A screen uses it to ask the router to show a
  different screen or to run a task for it.

- **`Router`**: holds the screens and the `History`. It owns the shared state.
  Build one with `Router::builder`.

- **`Message`**: is the router's message type. Widgets produce `Message::GoTo(id)`
  to navigate. The router uses `Message::Screen` internally to deliver a message to
  a specific screen. You never need to build one yourself.

- **`Navigation`**: sets the history behaviour. `Stack` keeps a back stack and
  `Back` goes back. `Switch` keeps only the current screen and `Back` does nothing.
  Screens don't need to change.

## Shared state

Screens can share data through a common state passed to them by the router.

```rust
impl Screen<Id, Shared> for Settings {
    fn update(&mut self, shared: &mut Shared, message: Message) -> Action<Id, Message> {
        match message {
            Message::Save => {
                shared.username = self.draft.clone();
                Action::None
            }
            // ...
        }
    }
}
```

Access it from outside by calling `Router::shared`, `Router::shared_mut`, and `Router::replace_shared`.

## Screens that keep state, and screens that don't

`.screen` builds a screen that lives for the lifetime of the router. Its state is kept on every visit.

`.route` builds a screen from the values in its id, when the router first goes there. It is dropped 
once its id leaves history.

```rust
Router::builder(Id::Home, ())
    .screen(Id::Home, home::Home::default())
    .route(|id| match id {
        Id::Search { query, page } => Some(search::Search::new(query, *page).boxed()),
        Id::User { id } => Some(user::User::new(*id).boxed()),
        _ => None,
    })
    .build();
```

An id carries everything its screen needs, so arguments don't go through the shared state. Two 
different ids are two different screens, so `Back` returns to the earlier one with its own state.

## Layout around the router

`Router::view` returns an `Element`, so put it wherever the content belongs. A navbar produces 
`Message::GoTo` directly, no mapping is required.

```rust
fn view(&self) -> Element<'_, Message> {
    column![
        navbar(self.router.current()),
        container(self.router.view()).padding(24),
    ]
    .into()
}
```

Swap `column!` for `row!` and it's a sidebar.

## Tasks and subscriptions

A screen returns tasks in its own message type. The router tags each task with the screen's handle, 
so the result comes back to the right screen even if the user has moved elsewhere.

```rust
Message::Refresh => Action::Run(Task::perform(fetch(), Message::Loaded)),
```

The router asks every live screen for a subscription, whether on show or not. A screen that want to 
stop a subscription returns `Subscription::none`.

## Nesting

A `Router` implements `Screen`, so you can nest routers. The inner router manages its own history, 
and passes `Action::Back` to the outer router only when it is already at its own root.

## Examples

| Example         | Shows                                                                      |
|-----------------|----------------------------------------------------------------------------|
| `01_quickstart` | `iced_router::application`, `Stack` navigation, `Action::Back`             |
| `02_navbar`     | Navbar outside the router, `Switch` navigation, shared state, `on_enter`   |
| `03_routes`     | Routes with arguments, route‑built screens, `on_enter` loading tasks       |

```
cargo run --example 01_quickstart
```

## Limits

- The screens and the routes are fixed when the router is built. A route builds
  and drops screens while the app runs, but you can't register a new one.

- Each screen has its own message type. A router erases a message's type so it can  
  hold them in a single list. If a router message is sent to the wrong screen is ignored.
  You don't have to worry about this because the router does the addressing. This only 
  matters if you build a `Message::Screen` by hand.

- The message type of a screen must be `Clone`. This is an iced rule: its widgets need it. 
  A screen id must be `Eq + Clone + Send + 'static`.

- A router inside another router only knows its own ids. A screen in the inner
  router cannot open a screen of the outer router. A screen inside a nested router only knows 
  its own router's id type, so it cannot navigate to a sibling of its parent. Put full screen 
  dialogs in the outer router.

- There is no way to block navigation. To stop a user reaching a screen, check the shared state 
  in your own `update`, before you return `Action::GoTo`.