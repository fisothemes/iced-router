use crate::ScreenHandle;
use std::any::Any;
use std::fmt;

/// A message that controls a [`Router`](crate::Router).
///
/// Widgets produce these to request navigation changes.
///
/// ```ignore
/// button("Settings").on_press(Message::GoTo(Id::Settings))
/// ```
///
/// - `I` is the identifier of a screen.
#[derive(Debug, Clone)]
pub enum Message<I> {
    /// Navigate to the screen with the given id.
    GoTo(I),
    /// Return to the previous screen.
    Back,
    /// Replace the current screen with the screen of the given id, without adding to history.
    Replace(I),
    /// Return to the root screen and drop all other history.
    Reset,
    /// A message addressed to a specific screen.
    ///
    /// The router generates these internally from screen views, tasks, and
    /// subscriptions. You never need to construct one yourself.
    ///
    /// Use [`Router::id_of`](crate::Router::id_of) to get the screen's id
    /// from a [`ScreenHandle`] (useful when reading logs).
    Screen(ScreenHandle, ErasedMessage),
}

/// The [`message`](crate::Screen::Message) of a [`Screen`](crate::Screen), with its type
/// forgotten.
pub struct ErasedMessage {
    message: Box<dyn Any + Send>,
    duplicate: fn(&(dyn Any + Send)) -> Box<dyn Any + Send>,
}

impl ErasedMessage {
    /// Erases the type of a message.
    pub fn new<M>(message: M) -> Self
    where
        M: Clone + Send + 'static,
    {
        Self {
            message: Box::new(message),
            duplicate: duplicate::<M>,
        }
    }

    /// Reads the message if it belongs to `M`.
    pub fn read<M>(self) -> Option<M>
    where
        M: Clone + Send + 'static,
    {
        self.message.downcast::<M>().ok().map(|message| *message)
    }
}

impl Clone for ErasedMessage {
    fn clone(&self) -> Self {
        Self {
            message: (self.duplicate)(&*self.message),
            duplicate: self.duplicate,
        }
    }
}

impl fmt::Debug for ErasedMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Erased(..)")
    }
}

fn duplicate<M>(message: &(dyn Any + Send)) -> Box<dyn Any + Send>
where
    M: Clone + Send + 'static,
{
    let message = message
        .downcast_ref::<M>()
        .expect("an erased message keeps the type it was made from");

    Box::new(message.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Hello(u32);

    #[derive(Debug, Clone, PartialEq)]
    struct Goodbye;

    #[test]
    fn a_message_survives_a_round_trip() {
        let erased = ErasedMessage::new(Hello(7));

        assert_eq!(erased.read::<Hello>(), Some(Hello(7)));
    }

    #[test]
    fn a_message_of_another_type_is_not_read() {
        let erased = ErasedMessage::new(Hello(7));

        assert_eq!(erased.read::<Goodbye>(), None);
    }

    #[test]
    fn a_clone_keeps_the_type_and_the_value() {
        let erased = ErasedMessage::new(Hello(7));
        let clone = erased.clone();

        assert_eq!(erased.read::<Hello>(), Some(Hello(7)));
        assert_eq!(clone.read::<Hello>(), Some(Hello(7)));
    }
}
