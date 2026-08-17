use crate::{Message, Router};
use iced::{Element, Subscription, Task};

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
/// ```no_run
/// # use iced::Element;
/// # use iced::Task;
/// # use iced_router::{Action, Message, Router, Screen};
/// #
/// # #[derive(Debug, Clone, PartialEq, Eq)]
/// # enum Id {
/// #     Home,
/// #     About,
/// # }
/// #
/// # #[derive(Debug, Clone)]
/// # enum ScreenMessage {}
/// #
/// # #[derive(Default)]
/// # struct Page;
/// #
/// # impl Screen<Id> for Page {
/// #     type Message = ScreenMessage;
/// #
/// #     fn update(
/// #         &mut self,
/// #         _shared: &mut (),
/// #         message: ScreenMessage,
/// #     ) -> Action<Id, ScreenMessage> {
/// #         match message {}
/// #     }
/// #
/// #     fn view<'a>(&'a self, _shared: &'a ()) -> Element<'a, ScreenMessage> {
/// #         iced::widget::space::horizontal().into()
/// #     }
/// # }
/// #
/// fn boot() -> (Router<Id>, (), Task<Message<Id>>) {
///     Router::builder(Id::Home)
///         .screen(Id::Home, Page::default())
///         .screen(Id::About, Page::default())
///         .build(())
/// }
///
/// fn main() -> iced::Result {
///     iced_router::application(boot).title("Quick start").run()
/// }
/// ```
pub fn application<I, S>(
    boot: impl Fn() -> (Router<I, S>, S, Task<Message<I>>) + 'static,
) -> iced::Application<impl iced::Program<Message = Message<I>, Theme = iced::Theme>>
where
    I: Eq + Clone + Send + 'static,
    S: 'static,
{
    iced::application(
        move || {
            let (router, shared, task) = boot();

            (Instance { router, shared }, task)
        },
        Instance::update,
        Instance::view,
    )
    .subscription(Instance::subscription)
    .title(Instance::title)
}

/// A [`Router`] and the state it is handed.
///
/// The router does not own the shared state, so something has to. For a single
/// window that something is this, and it is private because there is nothing
/// to configure on it.
struct Instance<I, S> {
    router: Router<I, S>,
    shared: S,
}

impl<I, S> Instance<I, S>
where
    I: Eq + Clone + Send + 'static,
{
    fn update(&mut self, message: Message<I>) -> Task<Message<I>> {
        self.router.update(&mut self.shared, message)
    }

    fn view(&self) -> Element<'_, Message<I>> {
        self.router.view(&self.shared)
    }

    fn subscription(&self) -> Subscription<Message<I>> {
        self.router.subscription(&self.shared)
    }

    fn title(&self) -> String {
        self.router.title(&self.shared).unwrap_or_default()
    }
}
