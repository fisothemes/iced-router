use crate::{Message, Router};
use iced::Task;

/// Runs a [`Router`] as an iced app.
///
/// You don't need to put the router inside a custom `App` struct.
/// Iced uses the `Router` directly as your application's state.
///
/// `boot` creates the router and returns it with a starting task.
/// Iced calls `boot` once at start, and again if the app restarts.
///
/// Each screen can set its own window title via [`Screen::title`](crate::Screen::title).
/// If a screen returns `None`, the title does not change.
///
/// Use this helper when the router fills the whole window.
/// If you have a navbar or sidebar, own the router yourself and use
/// [`iced::application`] directly.
///
/// ```ignore
/// fn main() -> iced::Result {
///     iced_router::application(boot).title("Quick start").run()
/// }
///
/// fn boot() -> (Router<Id>, Task<iced_router::Message<Id>>) {
///     Router::builder(Id::Home, ())
///         .screen(Id::Home, home::Home)
///         .screen(Id::About, about::About)
///         .build()
/// }
/// ```
pub fn application<I, S>(
    boot: impl Fn() -> (Router<I, S>, Task<Message<I>>) + 'static,
) -> iced::Application<impl iced::Program<Message = Message<I>, Theme = iced::Theme>>
where
    I: Eq + Clone + Send + 'static,
    S: 'static,
{
    iced::application(boot, Router::update, Router::view)
        .subscription(Router::subscription)
        .title(title)
}

fn title<I, S>(router: &Router<I, S>) -> String
where
    I: Eq + Clone + Send + 'static,
{
    router.title().unwrap_or_default()
}
