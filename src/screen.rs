use crate::Action;
use crate::ErasedMessage;
use iced::{Element, Subscription, Task};

/// A single screen of an application.
///
/// - `I` is the identifier type for screens (used in navigation actions).
/// - `S` is shared state that all screens can read and write.
pub trait Screen<I, S = ()> {
    /// The messages the screen produces and handles.
    type Message: Clone + Send + 'static;

    /// Called when the router navigates to the screen.
    ///
    /// The router keeps screens alive, so use this to reset the state or load
    /// shared data for each visit.
    fn on_enter(&mut self, shared: &mut S) -> Task<Self::Message> {
        let _ = shared;

        Task::none()
    }

    /// The window title when the screen is shown.
    ///
    /// Return `None` to keep the current title.
    fn title(&self, shared: &S) -> Option<String> {
        let _ = shared;

        None
    }

    /// Handles a message and returns what the router should do next.
    fn update(&mut self, shared: &mut S, message: Self::Message) -> Action<I, Self::Message>;

    /// Draws the screen.
    fn view<'a>(&'a self, shared: &'a S) -> Element<'a, Self::Message>;

    /// Handles subscription the subscription logic of the screen.
    fn subscription(&self, shared: &S) -> Subscription<Self::Message> {
        let _ = shared;

        Subscription::none()
    }

    /// Called when the router navigates away from the screen.
    ///
    /// Use this to stop tasks or clean up.
    fn on_exit(&mut self, shared: &mut S) -> Task<Self::Message> {
        let _ = shared;

        Task::none()
    }

    /// Converts the current screen into a boxed screen.
    fn boxed(self) -> BoxedScreen<I, S>
    where
        Self: Sized + 'static,
    {
        BoxedScreen::new(self)
    }
}

/// A trait that lets a screen be used as a trait object.
///
/// Methods take and return erased messages, so the concrete [`Message`](Screen::Message) type
/// is hidden. [`AnyScreen`] is implemented automatically for every [`Screen`].
pub trait AnyScreen<I, S> {
    fn on_enter(&mut self, shared: &mut S) -> Task<ErasedMessage>;

    fn title(&self, shared: &S) -> Option<String>;

    fn update(&mut self, shared: &mut S, message: ErasedMessage) -> Action<I, ErasedMessage>;

    fn view<'a>(&'a self, shared: &'a S) -> Element<'a, ErasedMessage>;

    fn subscription(&self, shared: &S) -> Subscription<ErasedMessage>;

    fn on_exit(&mut self, shared: &mut S) -> Task<ErasedMessage>;
}

impl<T, I, S> AnyScreen<I, S> for T
where
    T: Screen<I, S>,
{
    fn on_enter(&mut self, shared: &mut S) -> Task<ErasedMessage> {
        Screen::on_enter(self, shared).map(erase)
    }

    fn title(&self, shared: &S) -> Option<String> {
        Screen::title(self, shared)
    }

    fn update(&mut self, shared: &mut S, message: ErasedMessage) -> Action<I, ErasedMessage> {
        let Some(message) = message.read::<T::Message>() else {
            return Action::None;
        };

        Screen::update(self, shared, message).map(erase)
    }

    fn view<'a>(&'a self, shared: &'a S) -> Element<'a, ErasedMessage> {
        Screen::view(self, shared).map(erase)
    }

    fn subscription(&self, shared: &S) -> Subscription<ErasedMessage> {
        Screen::subscription(self, shared).map(erase::<T::Message>)
    }

    fn on_exit(&mut self, shared: &mut S) -> Task<ErasedMessage> {
        Screen::on_exit(self, shared).map(erase)
    }
}
fn erase<M>(message: M) -> ErasedMessage
where
    M: Clone + Send + 'static,
{
    ErasedMessage::new(message)
}

/// A concrete type that holds a trait object of [`AnyScreen`].
///
/// Use this when you need to store or return screens of different message types.
pub struct BoxedScreen<I, S = ()>(Box<dyn AnyScreen<I, S>>);

impl<I, S> BoxedScreen<I, S> {
    /// Wraps any [`Screen`] into a `BoxedScreen` by erasing its message type.
    pub fn new(screen: impl Screen<I, S> + 'static) -> Self {
        Self(Box::new(screen))
    }

    /// Unwraps the box, returning the trait object.
    pub fn into_inner(self) -> Box<dyn AnyScreen<I, S>> {
        self.0
    }
}

impl<I, S> AsRef<dyn AnyScreen<I, S>> for BoxedScreen<I, S> {
    fn as_ref(&self) -> &(dyn AnyScreen<I, S> + 'static) {
        self.0.as_ref()
    }
}

impl<I, S> AsMut<dyn AnyScreen<I, S>> for BoxedScreen<I, S> {
    fn as_mut(&mut self) -> &mut (dyn AnyScreen<I, S> + 'static) {
        self.0.as_mut()
    }
}

/// The address of a screen inside a [`Router`](crate::Router).
///
/// A unique handle given to each screen when it is created. This allows for tasks and
/// subscriptions find the right screen even if the screen was replaced.
///
/// # Important
///
/// The same handle should never be reused for a different screen, even if the screen is replaced.
///
/// Use a screen identifier to navigate. A [`ScreenHandle`] is for addressing an existing
/// screen, not for requesting navigation.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScreenHandle(usize);

impl ScreenHandle {
    pub fn new(value: usize) -> Self {
        Self(value)
    }
}
