use iced::Task;
use std::fmt;

/// A request from a [`Screen`](crate::Screen) to the router that owns it.
///
/// A screen returns an `Action` from
/// [`Screen::update`](crate::Screen::update) instead of changing the history
/// itself. This keeps a screen usable in any router, whatever its navigation
/// model.
#[derive(Default)]
pub enum Action<I, M> {
    /// Do nothing.
    #[default]
    None,
    /// Run a [`Task`].
    ///
    /// The router tags the output of the task with the id of the screen that
    /// returned it, so the result arrives back at that screen even if the
    /// user has navigated somewhere else in the meantime.
    Run(Task<M>),
    /// Go to the screen with the given id.
    GoTo(I),
    /// Go to the previous screen.
    Back,
    /// Go to the screen with the given id without growing the history.
    Replace(I),
    /// Go back to the root screen and forget every other screen.
    Reset,
}

impl<I, M> Action<I, M> {
    /// Turns an `Action<I, M>` into an `Action<I, N>`.
    ///
    /// Use this to lift the action of an inner screen into the message type
    /// of an outer one.
    pub fn map<N>(self, f: impl Fn(M) -> N + Send + 'static) -> Action<I, N>
    where
        M: Send + 'static,
        N: Send + 'static,
    {
        match self {
            Self::None => Action::None,
            Self::Run(task) => Action::Run(task.map(f)),
            Self::GoTo(id) => Action::GoTo(id),
            Self::Back => Action::Back,
            Self::Replace(id) => Action::Replace(id),
            Self::Reset => Action::Reset,
        }
    }

    /// Turns an `Action<I, M>` into an `Action<J, M>`.
    ///
    /// Use this when an inner router uses a different id type to the router
    /// that owns it.
    pub fn map_id<J>(self, f: impl FnOnce(I) -> J) -> Action<J, M> {
        match self {
            Self::None => Action::None,
            Self::Run(task) => Action::Run(task),
            Self::GoTo(id) => Action::GoTo(f(id)),
            Self::Back => Action::Back,
            Self::Replace(id) => Action::Replace(f(id)),
            Self::Reset => Action::Reset,
        }
    }

    /// Whether the action does nothing.
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl<I, M> fmt::Debug for Action<I, M>
where
    I: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("None"),
            Self::Run(_) => f.write_str("Run(..)"),
            Self::GoTo(id) => f.debug_tuple("GoTo").field(id).finish(),
            Self::Back => f.write_str("Back"),
            Self::Replace(id) => f.debug_tuple("Replace").field(id).finish(),
            Self::Reset => f.write_str("Reset"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum Id {
        Home,
        Settings,
    }

    #[test]
    fn map_keeps_navigation() {
        let action: Action<Id, u32> = Action::GoTo(Id::Settings);

        assert!(matches!(
            action.map(|n: u32| n.to_string()),
            Action::GoTo(Id::Settings)
        ));
    }

    #[test]
    fn map_id_rewrites_the_target() {
        let action: Action<u8, u32> = Action::Replace(1);

        assert!(matches!(
            action.map_id(|_id| Id::Home),
            Action::Replace(Id::Home)
        ));
    }

    #[test]
    fn the_default_does_nothing() {
        let action: Action<Id, u32> = Action::default();

        assert!(action.is_none());
    }
}
