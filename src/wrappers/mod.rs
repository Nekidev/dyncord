//! Wrappers around Twilight types to provide a more ergonomic interface.
//!
//! This module is divided into two submodules:
//!
//! - [`actions`]: Builders for actions that can be performed by the bot, such as sending messages.
//! - [`types`]: Wrappers around core Twilight types, such as messages and embeds.

use thiserror::Error;
use twilight_model::id::Id;

pub mod actions;
pub mod types;

/// An error was returned from an internal [`twilight_http`] call.
#[derive(Debug, Error)]
pub enum TwilightError {
    #[error("An error occurred while sending the message: {0}")]
    Twilight(#[from] twilight_http::Error),

    #[error("An error occurred while parsing the response from the Discord API: {0}")]
    TwilightParsing(#[from] twilight_http::response::DeserializeBodyError),
}

/// Trait to improve ergonomics over IDs.
pub trait IntoId<T> {
    /// Converts the current value into a twilight ID.
    ///
    /// Returns:
    /// [`Id<T>`] - The twilight [`Id`].
    fn into_id(self) -> Id<T>;
}

impl<T> IntoId<T> for u64 {
    fn into_id(self) -> Id<T> {
        Id::new(self)
    }
}

impl<T> IntoId<T> for Id<T> {
    fn into_id(self) -> Id<T> {
        self
    }
}
