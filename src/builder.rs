use crate::{BoxedScreen, DEFAULT_LIMIT, History, Message, Navigation, Route, Router, Screen};
use iced::Task;

/// Builds a [`Router`].
///
/// There are two ways to register a screen.
///
/// Use [`screen`](Builder::screen) for screens that keep state across visits
/// (like tabs). Use [`route`](Builder::route) for screens built on demand
/// from an id (like a user profile with arguments).
///
/// - `I` is the identifier of a screen.
/// - `S` is shared state that all screens can read and write.
///
/// ```no_run
/// use iced_router::{Navigation, Router, Screen};
/// # use iced::Element;
/// # use iced_router::Action;
/// #
/// # #[derive(Debug, Default)]
/// # struct Shared;
/// #
/// # #[derive(Debug, Clone)]
/// # enum Message {}
/// #
/// # macro_rules! screen {
/// #     ($name:ident) => {
/// #         #[derive(Default)]
/// #         struct $name;
/// #
/// #         impl Screen<Id, Shared> for $name {
/// #             type Message = Message;
/// #
/// #             fn update(
/// #                 &mut self,
/// #                 _shared: &mut Shared,
/// #                 _message: Message,
/// #             ) -> Action<Id, Message> {
/// #                 todo!()
/// #             }
/// #
/// #             fn view<'a>(&'a self, _shared: &'a Shared) -> Element<'a, Message> {
/// #                 todo!()
/// #             }
/// #         }
/// #     };
/// # }
/// #
/// # screen!(Home);
/// # screen!(Settings);
/// # screen!(User);
/// # screen!(Search);
/// #
/// # impl User {
/// #     fn new(_id: u64) -> Self {
/// #         Self
/// #     }
/// # }
/// #
/// # impl Search {
/// #     fn new(_query: &str, _page: u32) -> Self {
/// #         Self
/// #     }
/// # }
/// #
///
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// enum Id {
///     Home,
///     Settings,
///     User { id: u64 },
///     Search { query: String, page: u32 },
/// }
///
/// let (router, task) = Router::builder(Id::Home, Shared::default())
///     .navigation(Navigation::Stack)
///     .screen(Id::Home, Home::default())
///     .screen(Id::Settings, Settings::default())
///     .route(|id| match id {
///         Id::User { id } => Some(User::new(*id).boxed()),
///         Id::Search { query, page } => Some(Search::new(query, *page).boxed()),
///         _ => None,
///     })
///     .build();
/// #
/// # let _ = (router, task);
/// ```
pub struct Builder<I, S = ()> {
    screens: Vec<(I, BoxedScreen<I, S>)>,
    routes: Vec<Route<I, S>>,
    root: I,
    shared: S,
    navigation: Navigation,
    limit: usize,
}

impl<I, S> Router<I, S>
where
    I: Eq + Clone + Send + 'static,
{
    /// Starts building a [`Router`] with `root` as the initial screen.
    pub fn builder(root: I, shared: S) -> Builder<I, S> {
        Builder {
            screens: Vec::new(),
            routes: Vec::new(),
            root,
            shared,
            navigation: Navigation::default(),
            limit: DEFAULT_LIMIT,
        }
    }
}

impl<I, S> Builder<I, S>
where
    I: Eq + Clone + Send + 'static,
{
    /// Adds a screen that lives as long as the router.
    ///
    /// The screen keeps its state across visits. Use [`Screen::on_enter`] to reset it.
    /// Adding another screen with the same `id` replaces the old one.
    pub fn screen(mut self, id: I, screen: impl Screen<I, S> + 'static) -> Self {
        self.screens.retain(|(other, _)| *other != id);
        self.screens.push((id, screen.boxed()));
        self
    }

    /// Adds a function that builds a screen from an id.
    ///
    /// Return `None` for unrecognised `id`s. Routes are tried in order after all
    /// static screens, so they won’t shadow a [`screen`](Self::screen) entry.
    ///
    /// Screens built this way are dropped when their `id` leaves history.
    pub fn route(mut self, route: impl Fn(&I) -> Option<BoxedScreen<I, S>> + 'static) -> Self {
        self.routes.push(Box::new(route));
        self
    }

    /// Sets the navigation model. Defaults to [`Navigation::Stack`].
    pub fn navigation(mut self, navigation: Navigation) -> Self {
        self.navigation = navigation;
        self
    }

    /// Sets the [`History`] size limit for stack navigation. Default is [`DEFAULT_LIMIT`].
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Builds the router and calls [`Screen::on_enter`] on the root screen.
    ///
    /// # Panics
    ///
    /// Panics if no screen or route supplies the root id.
    pub fn build(self) -> (Router<I, S>, Task<Message<I>>) {
        let history = History::new(self.root, self.navigation).with_limit(self.limit);
        let mut router = Router::from_parts(self.screens, self.routes, history, self.shared);
        let root = router.current().clone();

        assert!(
            router.ensure(&root),
            "a router needs a screen or a route for its root id"
        );

        let task = router.enter();

        (router, task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Action;
    use iced::Element;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Id {
        Home,
        Settings,
        User { id: u64 },
    }

    #[derive(Debug, Clone)]
    enum Event {}

    struct Page;

    impl Screen<Id> for Page {
        type Message = Event;

        fn update(&mut self, _shared: &mut (), message: Event) -> Action<Id, Event> {
            match message {}
        }

        fn view<'a>(&'a self, _shared: &'a ()) -> Element<'a, Event> {
            iced::widget::space::horizontal().into()
        }
    }

    #[test]
    fn a_built_router_starts_at_its_root() {
        let (router, _) = Router::builder(Id::Home, ())
            .screen(Id::Home, Page)
            .screen(Id::Settings, Page)
            .build();

        assert_eq!(router.current(), &Id::Home);
        assert_eq!(router.len(), 2);
        assert_eq!(router.history().navigation(), Navigation::Stack);
    }

    #[test]
    fn the_navigation_model_is_kept() {
        let (router, _) = Router::builder(Id::Home, ())
            .navigation(Navigation::Switch)
            .screen(Id::Home, Page)
            .build();

        assert_eq!(router.history().navigation(), Navigation::Switch);
    }

    #[test]
    fn a_route_can_supply_the_root() {
        let (router, _) = Router::builder(Id::User { id: 1 }, ())
            .route(|id| match id {
                Id::User { .. } => Some(Page.boxed()),
                _ => None,
            })
            .build();

        assert_eq!(router.current(), &Id::User { id: 1 });
        assert_eq!(router.len(), 1);
    }

    #[test]
    fn a_route_does_not_run_at_build_time() {
        let (router, _) = Router::builder(Id::Home, ())
            .screen(Id::Home, Page)
            .route(|id| match id {
                Id::User { .. } => Some(Page.boxed()),
                _ => None,
            })
            .build();

        assert_eq!(router.len(), 1);
        assert!(router.can_go_to(&Id::User { id: 3 }));
    }

    #[test]
    fn the_same_id_is_only_added_once() {
        let (router, _) = Router::builder(Id::Home, ())
            .screen(Id::Home, Page)
            .screen(Id::Home, Page)
            .build();

        assert_eq!(router.len(), 1);
    }

    #[test]
    #[should_panic(expected = "root")]
    fn a_router_without_a_root_screen_is_refused() {
        let _ = Router::builder(Id::Home, ())
            .screen(Id::Settings, Page)
            .build();
    }
}
