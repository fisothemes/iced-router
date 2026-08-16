use crate::{
    Action, AnyScreen, BoxedScreen, ErasedMessage, History, Message, Screen, ScreenHandle,
};
use iced::{Element, Subscription, Task};

/// A function that builds a screen for an id it recognises.
pub type Route<I, S> = Box<dyn Fn(&I) -> Option<BoxedScreen<I, S>>>;

/// A live screen and its address.
struct Entry<I, S> {
    handle: ScreenHandle,
    id: I,
    screen: Box<dyn AnyScreen<I, S>>,
    /// True for screens added with [`Builder::screen`](crate::Builder::screen),
    /// since they're kept forever.
    kept: bool,
}

/// A set of screens and a history of visited ones.
///
/// The recommended way to build one is to use [`Router::builder`].
///
/// The router owns the shared state. Access it with [`shared`](Router::shared),
/// [`shared_mut`](Router::shared_mut), or [`replace_shared`](Router::replace_shared).
///
/// - `I` is the screen identifier.
/// - `S` is the shared state.
pub struct Router<I, S = ()> {
    screens: Vec<Entry<I, S>>,
    routes: Vec<Route<I, S>>,
    history: History<I>,
    shared: S,
    next: usize,
}

impl<I, S> Router<I, S>
where
    I: Eq + Clone + Send + 'static,
{
    /// Creates a router from the parts that [`Builder`](crate::Builder) collects.
    ///
    /// The caller must run [`Router::enter`] afterwards and ensure the root exists.
    pub fn from_parts(
        screens: Vec<(I, BoxedScreen<I, S>)>,
        routes: Vec<Route<I, S>>,
        history: History<I>,
        shared: S,
    ) -> Self {
        let screens: Vec<_> = screens
            .into_iter()
            .enumerate()
            .map(|(index, (id, screen))| Entry {
                handle: ScreenHandle::new(index),
                id,
                screen: screen.into_inner(),
                kept: true,
            })
            .collect();

        let next = screens.len();

        Self {
            screens,
            routes,
            history,
            shared,
            next,
        }
    }

    /// Runs [`Screen::on_enter`] for the current screen.
    ///
    /// [`Builder::build`](crate::Builder::build) already does this for the root.
    /// Call again only if you need to manually re‑enter.
    pub fn enter(&mut self) -> Task<Message<I>> {
        let id = self.history.current().clone();
        self.enter_id(&id)
    }

    /// Handles a router message.
    pub fn update(&mut self, message: Message<I>) -> Task<Message<I>> {
        match message {
            Message::GoTo(id) => self.go_to(id),
            Message::Back => self.back(),
            Message::Replace(id) => self.replace(id),
            Message::Reset => self.reset(),
            Message::Screen(handle, message) => {
                let Some(index) = self.index_of_handle(handle) else {
                    return Task::none();
                };

                let Self {
                    screens, shared, ..
                } = self;

                let action = screens[index].screen.update(shared, message);

                self.act(handle, action)
            }
        }
    }

    /// Draws the current screen.
    pub fn view(&self) -> Element<'_, Message<I>> {
        match self.index_of(self.history.current()) {
            Some(index) => {
                let entry = &self.screens[index];
                let handle = entry.handle;

                entry
                    .screen
                    .view(&self.shared)
                    .map(move |message| Message::Screen(handle, message))
            }
            None => iced::widget::space::horizontal().into(),
        }
    }

    /// Subscriptions from all live screens.
    pub fn subscription(&self) -> Subscription<Message<I>> {
        Subscription::batch(self.screens.iter().map(|entry| {
            entry
                .screen
                .subscription(&self.shared)
                .with(entry.handle)
                .map(|(handle, message)| Message::Screen(handle, message))
        }))
    }

    /// The title of the current screen.
    pub fn title(&self) -> Option<String> {
        let index = self.index_of(self.history.current())?;

        self.screens[index].screen.title(&self.shared)
    }

    /// The current screen's id.
    pub fn current(&self) -> &I {
        self.history.current()
    }

    /// Returns a reference to the router's history.
    pub fn history(&self) -> &History<I> {
        &self.history
    }

    /// Returns a reference to the shared state the screens.
    pub fn shared(&self) -> &S {
        &self.shared
    }

    /// Returns a mutable reference to the shared state of the screens.
    pub fn shared_mut(&mut self) -> &mut S {
        &mut self.shared
    }

    /// Replaces the shared state, returning the old one.
    pub fn replace_shared(&mut self, shared: S) -> S {
        std::mem::replace(&mut self.shared, shared)
    }

    /// Returns the id of a live screen, given its handle.
    pub fn id_of(&self, handle: ScreenHandle) -> Option<&I> {
        self.index_of_handle(handle)
            .map(|index| &self.screens[index].id)
    }

    /// Returns the handle of a live screen, given its id.
    pub fn handle_of(&self, id: &I) -> Option<ScreenHandle> {
        self.index_of(id).map(|index| self.screens[index].handle)
    }

    /// True if a screen with `id` is currently live.
    ///
    /// A screen added with [`Builder::screen`](crate::Builder::screen) is
    /// always live. A screen built via [`Builder::route`](crate::Builder::route) is live
    /// while its id is in [`history`](History).
    pub fn is_live(&self, id: &I) -> bool {
        self.index_of(id).is_some()
    }

    /// True if the router can navigate to a screen with a given `id`.
    ///
    /// Navigating to an `id` no route recognises is already ignored, so this is
    /// only needed to disable a link before the user presses it.
    ///
    /// # Performance
    ///
    /// This runs [`route`](crate::Builder::route) functions and discards the screen they return,
    /// so keep them cheap: build the state in the constructor and start the work
    /// in [`Screen::on_enter`].
    pub fn can_go_to(&self, id: &I) -> bool {
        self.is_live(id) || self.routes.iter().any(|route| route(id).is_some())
    }

    /// The number of live screens.
    pub fn len(&self) -> usize {
        self.screens.len()
    }

    /// True if the router holds no live screens.
    pub fn is_empty(&self) -> bool {
        self.screens.is_empty()
    }

    /// Makes sure a screen for `id` is live, building one from a route when it
    /// is not. Returns `true` if the router can show its `id`.
    pub fn ensure(&mut self, id: &I) -> bool {
        if self.is_live(id) {
            return true;
        }

        let Some(screen) = self.routes.iter().find_map(|route| route(id)) else {
            return false;
        };

        let handle = ScreenHandle::new(self.next);

        self.next += 1;

        self.screens.push(Entry {
            handle,
            id: id.clone(),
            screen: screen.into_inner(),
            kept: false,
        });

        true
    }

    fn act(&mut self, handle: ScreenHandle, action: Action<I, ErasedMessage>) -> Task<Message<I>> {
        match action {
            Action::None => Task::none(),
            Action::Run(task) => tag(handle, task),
            Action::GoTo(id) => self.go_to(id),
            Action::Back => self.back(),
            Action::Replace(id) => self.replace(id),
            Action::Reset => self.reset(),
        }
    }

    fn go_to(&mut self, id: I) -> Task<Message<I>> {
        if !self.ensure(&id) {
            return Task::none();
        }

        let from = self.history.current().clone();

        if !self.history.go_to(id) {
            return Task::none();
        }

        self.transition(from)
    }

    fn back(&mut self) -> Task<Message<I>> {
        let from = self.history.current().clone();

        if !self.history.back() {
            return Task::none();
        }

        self.transition(from)
    }

    fn replace(&mut self, id: I) -> Task<Message<I>> {
        if !self.ensure(&id) {
            return Task::none();
        }

        let from = self.history.current().clone();

        if !self.history.replace(id) {
            return Task::none();
        }

        self.transition(from)
    }

    fn reset(&mut self) -> Task<Message<I>> {
        let root = self.history.root().clone();

        if !self.ensure(&root) {
            return Task::none();
        }

        let from = self.history.current().clone();

        if !self.history.reset() {
            return Task::none();
        }

        self.transition(from)
    }

    /// Runs [`Screen::on_exit`] for the old screen, drops routed screens that left the
    /// history, then runs [`Screen::on_enter`] for the new screen.
    fn transition(&mut self, from: I) -> Task<Message<I>> {
        let to = self.history.current().clone();

        let exit = match self.index_of(&from) {
            Some(index) => {
                let Self {
                    screens, shared, ..
                } = self;

                let entry = &mut screens[index];
                let handle = entry.handle;

                tag(handle, entry.screen.on_exit(shared))
            }
            None => Task::none(),
        };

        self.prune();

        Task::batch([exit, self.enter_id(&to)])
    }

    fn enter_id(&mut self, id: &I) -> Task<Message<I>> {
        match self.index_of(id) {
            Some(index) => {
                let Self {
                    screens, shared, ..
                } = self;

                let entry = &mut screens[index];
                let handle = entry.handle;

                tag(handle, entry.screen.on_enter(shared))
            }
            None => Task::none(),
        }
    }

    /// Drops the screens built by a [`route`](crate::Builder::route) whose id has left the history.
    fn prune(&mut self) {
        let history = &self.history;

        self.screens
            .retain(|entry| entry.kept || history.iter().any(|id| *id == entry.id));
    }

    fn index_of(&self, id: &I) -> Option<usize> {
        self.screens.iter().position(|entry| entry.id == *id)
    }

    fn index_of_handle(&self, handle: ScreenHandle) -> Option<usize> {
        self.screens.iter().position(|entry| entry.handle == handle)
    }
}

/// A [`Router`] can be a [`Screen`] inside another router.
///
/// The inner router owns its own shared state. It passes [`Action::Back`] outward only
/// when its own history is exhausted.
impl<I, J, S, T> Screen<I, T> for Router<J, S>
where
    J: Eq + Clone + Send + 'static,
{
    type Message = Message<J>;

    fn on_enter(&mut self, _shared: &mut T) -> Task<Message<J>> {
        Router::enter(self)
    }

    fn title(&self, _shared: &T) -> Option<String> {
        Router::title(self)
    }

    fn update(&mut self, _shared: &mut T, message: Message<J>) -> Action<I, Message<J>> {
        if matches!(message, Message::Back) && !self.history.can_go_back() {
            return Action::Back;
        }

        Action::Run(Router::update(self, message))
    }

    fn view<'a>(&'a self, _shared: &'a T) -> Element<'a, Message<J>> {
        Router::view(self)
    }

    fn subscription(&self, _shared: &T) -> Subscription<Message<J>> {
        Router::subscription(self)
    }
}

fn tag<I>(handle: ScreenHandle, task: Task<ErasedMessage>) -> Task<Message<I>>
where
    I: Send + 'static,
{
    task.map(move |message| Message::Screen(handle, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Navigation;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Id {
        Home,
        Settings,
        User { id: u64 },
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Event {
        Go(Id),
        Leave,
        Home,
    }

    #[derive(Default)]
    struct Log {
        entered: Vec<String>,
        exited: Vec<String>,
    }

    struct Page(String);

    impl Screen<Id, Log> for Page {
        type Message = Event;

        fn on_enter(&mut self, log: &mut Log) -> Task<Event> {
            log.entered.push(self.0.clone());

            Task::none()
        }

        fn update(&mut self, _log: &mut Log, message: Event) -> Action<Id, Event> {
            match message {
                Event::Go(id) => Action::GoTo(id),
                Event::Leave => Action::Back,
                Event::Home => Action::Reset,
            }
        }

        fn view<'a>(&'a self, _log: &'a Log) -> Element<'a, Event> {
            iced::widget::space::horizontal().into()
        }

        fn on_exit(&mut self, log: &mut Log) -> Task<Event> {
            log.exited.push(self.0.clone());

            Task::none()
        }
    }

    fn router(navigation: Navigation) -> Router<Id, Log> {
        let (mut router, _) = Router::builder(Id::Home, Log::default())
            .navigation(navigation)
            .screen(Id::Home, Page(String::from("home")))
            .screen(Id::Settings, Page(String::from("settings")))
            .route(|id| match id {
                Id::User { id } => Some(Page(format!("user {id}")).boxed()),
                _ => None,
            })
            .build();

        // Forget the root's `on_enter`, so each test starts from a clean log.
        router.shared_mut().entered.clear();

        router
    }

    fn send(router: &mut Router<Id, Log>, id: &Id, event: Event) {
        let handle = router.handle_of(id).expect("the screen is live");

        let _ = router.update(Message::Screen(handle, ErasedMessage::new(event)));
    }

    #[test]
    fn a_screen_can_navigate() {
        let mut router = router(Navigation::Stack);

        send(&mut router, &Id::Home, Event::Go(Id::Settings));

        assert_eq!(router.current(), &Id::Settings);
    }

    #[test]
    fn navigating_runs_the_hooks() {
        let mut router = router(Navigation::Stack);

        send(&mut router, &Id::Home, Event::Go(Id::Settings));

        assert_eq!(router.shared().entered, vec![String::from("settings")]);
        assert_eq!(router.shared().exited, vec![String::from("home")]);
    }

    #[test]
    fn a_refused_move_runs_no_hooks() {
        let mut router = router(Navigation::Stack);

        send(&mut router, &Id::Home, Event::Leave);

        assert_eq!(router.current(), &Id::Home);
        assert!(router.shared().entered.is_empty());
        assert!(router.shared().exited.is_empty());
    }

    #[test]
    fn an_unknown_screen_is_not_visited() {
        let (mut router, _) = Router::builder(Id::Home, Log::default())
            .screen(Id::Home, Page(String::from("home")))
            .build();

        let _ = router.update(Message::GoTo(Id::Settings));

        assert_eq!(router.current(), &Id::Home);
    }

    #[test]
    fn a_message_for_a_screen_that_is_not_shown_is_handled() {
        let mut router = router(Navigation::Stack);

        send(&mut router, &Id::Home, Event::Go(Id::Settings));
        send(&mut router, &Id::Home, Event::Go(Id::User { id: 7 }));

        assert_eq!(router.current(), &Id::User { id: 7 });
    }

    #[test]
    fn a_route_builds_a_screen_on_entry() {
        let mut router = router(Navigation::Stack);

        assert_eq!(router.len(), 2);

        send(&mut router, &Id::Home, Event::Go(Id::User { id: 7 }));

        assert_eq!(router.len(), 3);
        assert_eq!(router.shared().entered, vec![String::from("user 7")]);
    }

    #[test]
    fn a_routed_screen_is_dropped_when_it_leaves_the_history() {
        let mut router = router(Navigation::Stack);

        send(&mut router, &Id::Home, Event::Go(Id::User { id: 7 }));

        assert_eq!(router.len(), 3);

        send(&mut router, &Id::User { id: 7 }, Event::Leave);

        assert_eq!(router.len(), 2);
        assert_eq!(router.current(), &Id::Home);
        assert_eq!(
            router.shared().exited,
            vec![String::from("home"), String::from("user 7")]
        );
    }

    #[test]
    fn a_routed_screen_lives_while_it_is_in_the_history() {
        let mut router = router(Navigation::Stack);

        send(&mut router, &Id::Home, Event::Go(Id::User { id: 7 }));

        let handle = router.handle_of(&Id::User { id: 7 }).expect("the screen");

        send(&mut router, &Id::User { id: 7 }, Event::Go(Id::Settings));

        assert_eq!(router.id_of(handle), Some(&Id::User { id: 7 }));
    }

    #[test]
    fn two_arguments_are_two_screens() {
        let mut router = router(Navigation::Stack);

        send(&mut router, &Id::Home, Event::Go(Id::User { id: 7 }));
        send(
            &mut router,
            &Id::User { id: 7 },
            Event::Go(Id::User { id: 9 }),
        );

        assert_eq!(router.len(), 4);
        assert_eq!(router.history().depth(), 3);
    }

    #[test]
    fn an_address_is_never_given_out_twice() {
        let mut router = router(Navigation::Stack);

        send(&mut router, &Id::Home, Event::Go(Id::User { id: 7 }));

        let first = router.handle_of(&Id::User { id: 7 }).expect("the screen");

        send(&mut router, &Id::User { id: 7 }, Event::Leave);
        send(&mut router, &Id::Home, Event::Go(Id::User { id: 7 }));

        let second = router.handle_of(&Id::User { id: 7 }).expect("the screen");

        assert_ne!(first, second);
        assert_eq!(router.id_of(first), None);
    }

    #[test]
    fn reset_returns_to_the_root() {
        let mut router = router(Navigation::Stack);

        send(&mut router, &Id::Home, Event::Go(Id::Settings));
        send(&mut router, &Id::Settings, Event::Home);

        assert_eq!(router.current(), &Id::Home);
        assert_eq!(router.history().depth(), 1);
    }

    #[test]
    fn switch_never_goes_back() {
        let mut router = router(Navigation::Switch);

        send(&mut router, &Id::Home, Event::Go(Id::Settings));
        send(&mut router, &Id::Settings, Event::Leave);

        assert_eq!(router.current(), &Id::Settings);
    }

    #[test]
    fn a_kept_screen_survives_a_switch() {
        let mut router = router(Navigation::Switch);

        send(&mut router, &Id::Home, Event::Go(Id::Settings));

        // Home left the history, but it was added with `screen`, so it is kept.
        assert_eq!(router.len(), 2);
        assert!(router.is_live(&Id::Home));
    }

    #[test]
    fn a_route_is_reachable_before_it_is_built() {
        let router = router(Navigation::Stack);

        assert!(!router.is_live(&Id::User { id: 7 }));
        assert!(router.can_go_to(&Id::User { id: 7 }));
    }

    #[test]
    fn the_shared_state_can_be_swapped() {
        let mut router = router(Navigation::Stack);

        send(&mut router, &Id::Home, Event::Go(Id::Settings));

        let previous = router.replace_shared(Log::default());

        assert!(router.shared().exited.is_empty());
        assert_eq!(previous.exited, vec![String::from("home")]);
    }
}
