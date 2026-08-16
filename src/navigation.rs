/// The way a [`History`](crate::History) handles navigation to a new screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Navigation {
    /// Navigating pushes the new screen onto the history.
    ///
    /// [`History::back`](crate::History::back) returns to the previous
    /// screen. Use this for drill-down interfaces and wizards.
    #[default]
    Stack,
    /// Navigating replaces the current screen.
    ///
    /// The history is always one screen deep and
    /// [`History::back`](crate::History::back) does nothing. Use this for
    /// navbars, tab bars, and sidebars.
    Switch,
}
