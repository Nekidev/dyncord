//! Error types for command running and argument parsing.
//!
//! These errors are generic across all command types, meaning that they're returned by all command
//! handlers via internal wrappers.
//!
//! There's two error types here, [`CommandError`] and [`ArgumentError`].
//!
//! - [`CommandError`] - Either an argument error or a runtime error. Runtime errors are boxed and
//!   type-erased.
//! - [`ArgumentError`] - An error occurred while parsing arguments. It also contains a `Runtime`
//!   variant due to support for custom argument parsers.
//!
//! When [`CommandError`]s happen, they're passed to any error handlers as
//! [`DyncordError::Command`](crate::errors::DyncordError). Match against it and read each error
//! type's documentation to know what happened and where to look at.
//! 
//! Error handling is documented at [the top-level `errors` module](crate::errors). That includes
//! the errors defined in this module.

use std::error::Error;
use std::sync::Arc;

/// An error that occurred when a command was called.
#[derive(Debug, thiserror::Error, Clone)]
pub enum CommandError {
    /// An error occurred while parsing a command's arguments.
    #[error("An error occurred while parsing a command's arguments: {0}")]
    Arguments(#[from] ArgumentError),

    /// An error occurred while running a command.
    ///
    /// These are handler-introduced errors. When this error is returned, either your command
    /// handler panicked or it returned an error.
    #[error("An error occurred while running a command: {0}")]
    Runtime(Arc<dyn Error + Send + Sync>),
}

/// An error that occurred while parsing a command's argument.
#[derive(Debug, thiserror::Error, Clone)]
pub enum ArgumentError {
    /// An argument was required, but it was missing.
    #[error("A required argument was missing.")]
    Missing,

    /// An argument did not have required metadata.
    ///
    /// This shouldn't happen, yet it's here to get to know when a bug occurs. This error means a
    /// bug on dyncord's side.
    #[error("An argument required (its own) metadata, but it was missing.")]
    MissingMeta,

    /// An argument was received, but it was improperly formatted.
    ///
    /// E.g. a number was required, but a non-numerical string was passed.
    #[error("An argument was improperly formatted.")]
    Misformatted,

    /// An argument was received, but the argument type received was not correct.
    ///
    /// This happens when a slash command with mismatching argument metadata is invoked. This
    /// shouldn't happen, and any appearance of this error means an error on dyncord's side.
    #[error("An argument was received with an incorrect type.")]
    Mistyped,

    /// An unknown error occurred while parsing an argument.
    ///
    /// This is reserved to developer-induced errors. If this error is returned, check your custom
    /// argument type parser. The error occurred there.
    #[error("An unknown error occurred while parsing an argument: {0}")]
    Runtime(Arc<dyn Error + Send + Sync>),
}
