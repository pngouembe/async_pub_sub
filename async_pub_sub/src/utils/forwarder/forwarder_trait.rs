use crate::{Publisher, Subscriber};

/// A trait for types that can both publish and subscribe to messages.
///
/// A `Forwarder` acts as both a `Publisher` and a `Subscriber`, allowing it to receive messages
/// and then potentially resend them or process them in some way before publishing them elsewhere.
pub trait Forwarder: Publisher + Subscriber {}
