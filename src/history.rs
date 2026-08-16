use crate::Navigation;
use std::collections::VecDeque;

/// The number of screens a [`Navigation::Stack`] history keeps by default.
pub const DEFAULT_LIMIT: usize = 64;

/// A history of visited screens.
///
/// A `History` always holds at least one screen, so [`History::current`]
/// always has something to return.
#[derive(Debug, Clone, PartialEq)]
pub struct History<I> {
    root: I,
    stack: VecDeque<I>,
    navigation: Navigation,
    limit: usize,
}

impl<I> History<I>
where
    I: PartialEq + Clone,
{
    /// Creates a [`History`] that starts at `root`.
    pub fn new(root: I, navigation: Navigation) -> Self {
        Self {
            stack: VecDeque::from([root.clone()]),
            root,
            navigation,
            limit: DEFAULT_LIMIT,
        }
    }

    /// Sets the number of screens the history keeps.
    ///
    /// When the history grows past `limit`, the oldest screens are dropped.
    /// A limit of 0 becomes 1. [`Navigation::Switch`] ignores this.
    ///
    /// Dropping the oldest screens can remove the root from the history, but
    /// [`History::reset`] still works because the root is stored separately.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit.max(1);
        self.trim();
        self
    }

    /// The screen to show.
    pub fn current(&self) -> &I {
        self.stack.back().expect("a history is never empty")
    }

    /// The screen the history started at.
    pub fn root(&self) -> &I {
        &self.root
    }

    /// The navigation model of the history.
    pub fn navigation(&self) -> Navigation {
        self.navigation
    }

    /// The number of screens in the history. Always 1 or more.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Whether [`History::back`] changes the current screen.
    pub fn can_go_back(&self) -> bool {
        self.navigation == Navigation::Stack && self.stack.len() > 1
    }

    /// The screens in the history, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = &I> {
        self.stack.iter()
    }

    /// Goes to `id`.
    ///
    /// [`Navigation::Stack`] pushes `id` onto the history.
    /// [`Navigation::Switch`] replaces the history with `id`.
    ///
    /// Returns whether the current screen changed. Going to the current
    /// screen does nothing.
    pub fn go_to(&mut self, id: I) -> bool {
        if *self.current() == id {
            return false;
        }

        match self.navigation {
            Navigation::Stack => {
                self.stack.push_back(id);
                self.trim();
            }
            Navigation::Switch => {
                self.stack.clear();
                self.stack.push_back(id);
            }
        }

        true
    }

    /// Goes to the previous screen.
    ///
    /// Returns whether the current screen changed. The root of a
    /// [`Navigation::Stack`] history and every [`Navigation::Switch`] history
    /// stay where they are.
    pub fn back(&mut self) -> bool {
        if !self.can_go_back() {
            return false;
        }

        self.stack.pop_back();

        true
    }

    /// Replaces the current screen.
    ///
    /// The depth of the history does not change, so the screen that was
    /// replaced cannot be returned to.
    ///
    /// Returns whether the current screen changed.
    pub fn replace(&mut self, id: I) -> bool {
        if *self.current() == id {
            return false;
        }

        *self.stack.back_mut().expect("a history is never empty") = id;

        true
    }

    /// Goes back to the root and forgets every other screen.
    ///
    /// Returns whether the current screen changed.
    pub fn reset(&mut self) -> bool {
        if self.stack.len() == 1 && *self.current() == self.root {
            return false;
        }

        self.stack.clear();
        self.stack.push_back(self.root.clone());

        true
    }

    fn trim(&mut self) {
        while self.stack.len() > self.limit {
            self.stack.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack() -> History<&'static str> {
        History::new("home", Navigation::Stack)
    }

    fn switch() -> History<&'static str> {
        History::new("home", Navigation::Switch)
    }

    #[test]
    fn starts_at_the_root() {
        let history = stack();

        assert_eq!(history.current(), &"home");
        assert_eq!(history.depth(), 1);
        assert!(!history.can_go_back());
    }

    #[test]
    fn stack_pushes() {
        let mut history = stack();

        assert!(history.go_to("settings"));
        assert_eq!(history.current(), &"settings");
        assert_eq!(history.depth(), 2);
        assert!(history.can_go_back());
    }

    #[test]
    fn stack_back_stops_at_the_root() {
        let mut history = stack();

        history.go_to("settings");
        history.go_to("about");

        assert!(history.back());
        assert!(history.back());
        assert!(!history.back());
        assert_eq!(history.current(), &"home");
    }

    #[test]
    fn going_to_the_current_screen_does_nothing() {
        let mut history = stack();

        history.go_to("settings");

        assert!(!history.go_to("settings"));
        assert_eq!(history.depth(), 2);
    }

    #[test]
    fn a_screen_can_be_visited_twice() {
        let mut history = stack();

        history.go_to("settings");
        history.go_to("home");

        assert_eq!(history.depth(), 3);
        assert_eq!(history.current(), &"home");

        history.back();

        assert_eq!(history.current(), &"settings");
    }

    #[test]
    fn switch_never_grows() {
        let mut history = switch();

        history.go_to("settings");
        history.go_to("about");

        assert_eq!(history.depth(), 1);
        assert_eq!(history.current(), &"about");
        assert!(!history.can_go_back());
        assert!(!history.back());
    }

    #[test]
    fn replace_keeps_the_depth() {
        let mut history = stack();

        history.go_to("settings");

        assert!(history.replace("about"));
        assert_eq!(history.depth(), 2);
        assert_eq!(history.current(), &"about");

        history.back();

        assert_eq!(history.current(), &"home");
    }

    #[test]
    fn replace_at_the_root_keeps_a_screen() {
        let mut history = stack();

        history.replace("settings");

        assert_eq!(history.depth(), 1);
        assert_eq!(history.current(), &"settings");
        assert!(!history.back());
    }

    #[test]
    fn reset_returns_to_the_root() {
        let mut history = stack();

        history.go_to("settings");
        history.go_to("about");

        assert!(history.reset());
        assert_eq!(history.depth(), 1);
        assert_eq!(history.current(), &"home");
        assert!(!history.reset());
    }

    #[test]
    fn reset_returns_to_the_root_after_replace() {
        let mut history = stack();

        history.replace("settings");

        assert!(history.reset());
        assert_eq!(history.current(), &"home");
    }

    #[test]
    fn the_limit_drops_the_oldest_screens() {
        let mut history = History::new(0, Navigation::Stack).with_limit(3);

        for id in 1..=5 {
            history.go_to(id);
        }

        assert_eq!(history.depth(), 3);
        assert_eq!(history.iter().copied().collect::<Vec<_>>(), vec![3, 4, 5]);
    }

    #[test]
    fn the_root_survives_the_limit() {
        let mut history = History::new(0, Navigation::Stack).with_limit(2);

        for id in 1..=5 {
            history.go_to(id);
        }

        assert!(history.reset());
        assert_eq!(history.current(), &0);
        assert_eq!(history.depth(), 1);
    }

    #[test]
    fn a_limit_of_zero_keeps_one_screen() {
        let mut history = History::new("home", Navigation::Stack).with_limit(0);

        history.go_to("settings");

        assert_eq!(history.depth(), 1);
        assert_eq!(history.current(), &"settings");
    }
}
