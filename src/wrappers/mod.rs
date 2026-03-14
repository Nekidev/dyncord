//! Wrappers around Twilight types to provide a more ergonomic interface.
//!
//! This module is divided into two submodules:
//!
//! - [`actions`]: Builders for actions that can be performed by the bot, such as sending messages.
//! - [`types`]: Wrappers around core Twilight types, such as messages and embeds.

use thiserror::Error;

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