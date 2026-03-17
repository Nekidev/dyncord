//! Error handlers, types, and contexts.
//!
//! Dyncord supports modular, layered error handling. For error handling to work properly, though,
//! you need to keep one rule in mind: **Do not let your code panic.** Panics break error handling,
//! so if you panic, God save you. Okay, maybe not God save you, but any handlers pending to run
//! for the current error queue won't run, which is not nice.
//!
//! # Quick Start
//!
//! Error handling is done the same way everything else is done on Dyncord: using async functions.
//! To define your first error handler, create the following function:
//!
//! ```
//! async fn handle_error(ctx: ErrorContext, error: DyncordError) {
//!     // Error-handling code here.
//! }
//! ```
//!
//! Every part of dyncord that allows you to pass a custom function as an argument to handle
//! something also supports error handling. For example, to handle bot-wide errors,
//! [`Bot`](crate::Bot) has a [`Bot::on_error`](crate::Bot::on_error) function that can be used one
//! or many times, once per error handler.
//!
//! Error handling is done in layers. Let's suppose we want to handle `MessageCreate` events.
//! However, what we're doing inside that handler is querying a database, and such query can fail.
//!
//! ```
//! #[derive(Debug, thiserror::Error)]
//! #[error("Oh no! An error occurred with the database: {0}")]
//! struct DatabaseError(&'static str);
//!
//! async fn on_message_create(ctx: EventContext<(), MessageCreate>) -> Result<(), DatabaseError> {
//!     Err(DatabaseError("We couldn't connect to the database. In fact, we didn't even try. Good luck."))
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let bot = Bot::new(())
//!         .intents(Intents::GUILD_MESSAGES)
//!         .on_event(On::message_create(on_message_create));
//!
//!     bot.run("token").await.unwrap();
//! }
//! ```
//!
//! > *Note: Yes! Event handlers, like command handlers and error handlers, can return an arbitrary
//! > `Result<T, E>`. `Err(E)` will fire error handlers.*
//!
//! When we run our bot, events will start flowing in. Our event handler fails on every call, but
//! they go unnoticed and that sucks. Let's add the error handler we created in the first example.
//!
//! ```
//! async fn on_error(ctx: ErrorContext, error: DyncordError) {
//!     println!("Oh no! An error occurred: {error}");
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let bot = Bot::new(())
//!         .intents(Intents::GUILD_MESSAGES)
//!         .on_event(On::message_create(on_message_create))
//!         .on_error(on_error);
//!
//!     bot.run("token").await.unwrap();
//! }
//! ```
//!
//! Run your code again, and you'll see that whenever an event comes in, the error handler gets
//! fired and the error is logged. Great.
//!
//! Now, what if we add a prefixed command to our bot that also fails every time? Let's try it out.
//!
//! ```
//! #[derive(Debug, thiserror::Error)]
//! #[error("Oh no! A completely-unexpected error occurred when the user run the fail command: {0}")]
//! struct FailError(String);
//!
//! async fn fail(ctx: PrefixedContext, message: String) -> Result<(), FailError> {
//!     Err(FailError(message))
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let bot = Bot::new(())
//!         .with_prefix(".")
//!         .intents(Intents::GUILD_MESSAGES)
//!         .intents(Intents::MESSAGE_CONTENT)
//!         .command(Command::prefixed("fail", fail))
//!         .on_event(On::message_create(on_message_create))
//!         .on_error(on_error);
//!
//!     bot.run("token").await.unwrap();
//! }
//! ```
//!
//! Rerun your bot and try the `.fail` command. You'll see it get logged successfully. What if you
//! run `.fail` without any arguments? It'll also get logged, this time as a command error.
//!
//! # Scoped Error Handling
//!
//! This is working well, it logs everything, but what if we want to have an error handler that
//! only runs on command errors? We want to handle `.fail`'s errors differently than the rest.
//!
//! Let's make our `on_error` handler only handle the `.fail` command's errors.
//!
//! ```
//! let bot = Bot::new(())
//!     .with_prefix(".")
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .command(Command::prefixed("fail", fail).on_error(on_error))  // Move the .on_error() to this line.
//!     .on_event(On::message_create(on_message_create));
//!
//! bot.run("token").await.unwrap();
//! ```
//!
//! Try running your bot again. You'll see that now, errors returned by the event handler no longer
//! get logged but `.fail` ones still do. Good!
//!
//! You can do the same with the event handler. Let's move our error handler to the event handler.
//!
//! ```
//! let bot = Bot::new(())
//!     .with_prefix(".")
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .command(Command::prefixed("fail", fail).on_error(on_error))
//!     .on_event(On::message_create(on_message_create).on_error(on_error));  // Add the .on_error() to this line.
//!
//! bot.run("token").await.unwrap();
//! ```
//!
//! Rerun your bot and you'll see that now error logging is back to how we started. You can add
//! error handlers to specific commands, command groups, events, and bot-wide. The same handlers
//! can be recycled, too.
//!
//! # Failing Error Handlers and Layers
//!
//! Even error handlers can fail. What when they do? Are they immune to error handling? In dyncord,
//! handlers can catch errors raised by more-specific error handlers, and also catch errors that
//! more-specific error handlers failed to handle.
//!
//! For example, we may want to store error logs in a database. Such thing, as in our previous
//! example, can fail. In such case, we want to just log the error onto the terminal.
//!
//! Our first error handler looks like this:
//!
//! ```
//! #[derive(Debug, thiserror::Error)]
//! #[error("Oh no! A completely-unexpected error occurred when the user run the fail command: {0}")]
//! struct FailError(String);
//!
//! #[derive(Debug, thiserror::Error)]
//! #[error("Oh no! An error occurred with the database: {0}")]
//! struct DatabaseError(&'static str);
//!
//! async fn fail(ctx: PrefixedContext, message: String) -> Result<(), FailError> {
//!     Err(FailError(message))
//! }
//!
//! async fn log_to_database(_ctx: ErrorContext, _error: DyncordError) -> Result<(), DatabaseError> {
//!     Err(DatabaseError("Oh no! We didn't try to save anything, and somehow nothing got saved. An error? Are we getting hacked? President is it you?"))
//! }
//!
//! let bot = Bot::new(())
//!     .with_prefix(".")
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .command(Command::prefixed("fail", fail).on_error(log_to_database));
//!
//! bot.run("token").await.unwrap();
//! ```
//!
//! Great. But if you look closely, you'll see that our `log_to_database` error handler never
//! succeeds for mysterious reasons. That means that now we have errors silently occurring and
//! disappearing.
//!
//! As we said, we want to log to the terminal when we can't log to the database. Let's make a
//! simple error handler to do such thing.
//!
//! ```
//! async fn log_to_terminal(_ctx: ErrorContext, error: DyncordError) {
//!     println!("ERORR: {error}");
//! }
//! ```
//!
//! Now, let's make that one a [`Bot`](crate::Bot)-level error handler.
//!
//! ```
//! #[derive(Debug, thiserror::Error)]
//! #[error("Oh no! A completely-unexpected error occurred when the user run the fail command: {0}")]
//! struct FailError(String);
//!
//! #[derive(Debug, thiserror::Error)]
//! #[error("Oh no! An error occurred with the database: {0}")]
//! struct DatabaseError(&'static str);
//!
//! async fn fail(ctx: PrefixedContext, message: String) -> Result<(), FailError> {
//!     Err(FailError(message))
//! }
//!
//! async fn log_to_database(_ctx: ErrorContext, _error: DyncordError) -> Result<(), DatabaseError> {
//!     Err(DatabaseError("Oh no! We didn't try to save anything, and somehow nothing got saved. An error? Are we getting hacked? President is it you?"))
//! }
//!
//! async fn log_to_terminal(_ctx: ErrorContext, error: DyncordError) {
//!     println!("ERORR: {error}");
//! }
//!
//! let bot = Bot::new(())
//!     .with_prefix(".")
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .command(Command::prefixed("fail", fail).on_error(log_to_database))
//!     .on_error(log_to_terminal);  // Add this line.
//!
//! bot.run("token").await.unwrap();
//! ```
//!
//! Try running your bot and running the `.fail` command. You'll see two errors being logged onto
//! the terminal. Well done.
//!
//! If you're asking yourself why two errors, the answer is simple: `FailError` failed to be
//! logged into the database, but we don't want it disappearing because of it. It's passed to the
//! next most-specific error handler there is, in this case `log_to_terminal`. `DatabaseError` also
//! has to be handled, so it's also passed down to `log_to_terminal`. That makes it two log lines.
//!
//! As stated before, dyncord's error handling is layered. Any errors not handled by the first
//! layer will be passed down to the next available layer, and if such layer fails to handle it,
//! it'll be passed down to the next layer until either the error gets handled or no more layers
//! are available to handle it.
//!
//! Layers go from most-specific to least-specific error handler. A bot-wide error handler is
//! less specific than a command-specific error handler, and a group-wide error handler is more
//! specific than a bot-wide error handler but less-specific than a command-specific error handler.
//!
//! # Intentionally Ignoring Errors
//!
//! Usually, you may find yourself making error handlers for only a specific kind or group of
//! errors. You want to let some errors pass down, but it's not an error to do so so you don't want
//! to return a custom error to force the error to be handled by a less-specific layer. That would
//! also introduce a special case not to handle your custom "pass down this error" error, it gets
//! ugly fast.
//!
//! Luckily, dyncord has a built-in way of doing this while keeping your happiness intact. Make the
//! error handler return `Result<T, ErrorHandlerError>`, and when you don't want to handle an
//! error, just return [`ErrorHandlerError::NotHandled`].
//!
//! ```
//! /// An error handler that does not handle any errors.
//! async fn on_error_not_handle(ctx: ErrorContext, error: DyncordError) -> Result<(), ErrorHandlerError> {
//!     Err(ErrorHandlerError::NotHandled)
//! }
//! ```
//!
//! If you add a logging error handler on a layer lower to that, you'll see that the original error
//! gets logged but no `ErrorHandlerError` log can be seen. That's because
//! [`ErrorHandlerError::NotHandled`] means you chose not to handle such error, and therefore it's
//! not *actually* an error.
//!
//! # Multi-Handler Error Handler Layers
//!
//! Dyncord also supports having multiple error handlers per layer. In such cases, when an error
//! reaches a layer, **all error handlers in such layer are called with the error**.
//!
//! To define multiple error handlers per layer, just chain `.on_error()` calls. For example,
//!
//! ```
//! async fn handler_1(ctx: ErrorContext, error: DyncordError) {
//!     println!("Handler 1 is running... {error}");
//! }
//!
//! async fn handler_2(ctx: ErrorContext, error: DyncordError) {
//!     println!("Handler 2 is running... {error}");
//! }
//!
//! let bot = Bot::new(())
//!     .on_error(handler_1)
//!     .on_error(handler_2);
//! ```
//!
//! Whenever an error reaches the bot-wide layer, both error handlers will be called. It doesn't
//! matter whether one of those succeeds.
//!
//! When more than one handler is in a layer, for an error to be passed to the next layer no
//! handler in the current layer must have succeeded to handle the error.
//!
//! For example, in a more complex event handler situation, events could be passed down as follows:
//!
//! ```text
//! handler_1(error) -> Always fails
//! handler_2(error) -> Always doesn't handle error
//! handler_3(error) -> Always succeeds only on command errors
//! handler_4(error) -> Always fails
//! handler_5(error) -> Always fails
//!
//! command() -> Error
//!
//! handle(ctx, error, [[handler_1, handler_2], [handler_3, handler_4], [handler_5]]);
//!
//! handler_1(error) -> Fails
//! handler_2(error) -> Didn't handle
//!
//! handler_3(handler_1_error) -> Didn't handle
//! handler_3(error) -> Succeess!
//! handler_4(handler_1_error) -> Fails
//! handler_4(error) -> Fails
//!
//! handler_5(handler_1_error) -> Fails
//! handler_5(handler_4_error_1) -> Fails
//! handler_5(handler_4_error_2) -> Fails
//! ```
//!
//! As you can see, `handler_1`'s error gets passed all the way down because no handler in the
//! middle layers handles it properly. The original error gets properly handled by `handler_3`, so
//! that error stops getting propagated. `handler_4` raises an error twice, one per call, so
//! `handler_5` ends up handling `handler_1`'s error and both of `handler_4`'s errors. `handler_5`
//! fails, but there's no more error handler layers so 6 errors go unhandled.
//!
//! # Downcasting to Custom Errors
//!
//! All error handlers are required to take `DyncordError` as an error. That's useful for basic
//! logging, but what when you want to act differently based on the error type you actually
//! returned?
//!
//! [`DyncordError`] has a [`downcast()`](DyncordError::downcast) associated function that does the
//! error pattern matching and downcasting for you all in one go, without needing to manually match
//! and try to downcast on every possible variant. Spoiler: [`DyncordError`] gets quite nested.
//!
//! Downcasting is simple, take the following example
//!
//! ```
//! #[derive(Debug, thiserror::Error)]
//! #[error("Oh no! A completely-unexpected error occurred when the user run the fail command: {0}")]
//! struct FailError(String);
//!
//! async fn fail(ctx: PrefixedContext, message: String) -> Result<(), FailError> {
//!     Err(FailError(message))
//! }
//!
//! async fn on_fail_error(_ctx: ErrorContext, error: DyncordError) {
//!     if let Some(error) = error.downcast::<FailError>() {  // Here.
//!         println!("ERROR: {}", error.0);
//!     }
//! }
//!
//! let bot = Bot::new(())
//!     .with_prefix(".")
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .command(Command::prefixed("fail", fail).on_error(on_fail_error));
//!
//! bot.run("token").await.unwrap();
//! ```
//!
//! Given you're allowed to return any error type you want from your handlers, dyncord has to erase
//! the type from your error to store it into a generic [`DyncordError`]. Downcasting here attempts
//! to get the type-erased error and see if it matches the error type you gave it. If it does, then
//! it returns the value to you in the `Some()`.
//!
//! # Original Error Contexts
//!
//! Many times, you want to let the user know an error happened. However, [`ErrorContext`] is
//! neither [`PrefixedContext`] nor [`SlashContext`], so you can't directly tell the user an error
//! occurred.
//!
//! [`ErrorContext`] has an [`original`](ErrorContext::original) field that holds the original
//! context type. You could match against it to tell the user an error occurred.
//!
//! ```
//! async fn on_error_notify_user(ctx: ErrorContext, _error: DyncordError) -> Result<(), ErrorHandlerError> {
//!     match ctx.original {
//!         ErrorOriginalContext::PrefixedContext(ctx) => {
//!             let _ = ctx.send("Oh no! An error occurred.").await.map_err(ErrorHandlerError::new)?;
//!         },
//!         ErrorOriginalContext::SlashContext(ctx) => {
//!             ctx.respond("Oh no! An error occurred.").await.map_err(ErrorHandlerError::new)?;
//!         },
//!         _ => {
//!             return Err(ErrorHandlerError::NotHandled)
//!         }
//!     };
//!
//!     Ok(())
//! }
//! ```
//! 
//! However, that's ugly. That's why you can use [`ErrorContext::send`] instead, which is similar
//! to the example above but cleaner.
//! 
//! ```
//! async fn on_error_notify_user(ctx: ErrorContext, _error: DyncordError) -> Result<(), ErrorHandlerError> {
//!     ctx.send("Oh no! An error occurred.").await?;
//!
//!     Ok(())
//! }
//! ```
//! 
//! [`ErrorContext::send`] is the same as [`PrefixedContext::send`] when the error was returned by
//! a prefixed-command handler, and the same as [`SlashContext::respond`] when the error was
//! returned by a slash-command handler.
//! 
//! The only difference with the first example matching on `ctx.original` is that when the message
//! is not sent, it doesn't return [`ErrorHandlerError::NotHandled`], rather `Ok(false)`.

use std::any::Any;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;

use twilight_gateway::Event;

use crate::commands::errors::{ArgumentError, CommandError};
use crate::commands::prefixed::context::PrefixedContext;
use crate::commands::prefixed::prefixes::PrefixesContext;
use crate::commands::slash::context::SlashContext;
use crate::events::EventContext;
use crate::handle::Handle;
use crate::state::StateBound;
use crate::utils::DynFuture;

/// A top-level error type for handle-able errors that occur while the bot is running.
#[derive(Debug, thiserror::Error, Clone)]
pub enum DyncordError {
    /// An error occurred when a command was invoked.
    #[error("An error occurred while running, or attempting to run, a command: {0}")]
    Command(#[from] CommandError),

    #[error("An error occurred while running an event handler: {0}")]
    Event(Arc<dyn Error + Send + Sync>),

    #[error("An error occurred while running an error handler: {0}")]
    Error(Arc<dyn Error + Send + Sync>),
}

impl DyncordError {
    /// Attempts to downcast a generic [`DyncordError`] to a specific type returned during runtime.
    ///
    /// For example, in an error handler:
    ///
    /// ```
    /// async fn on_specific_error(_ctx: ErrorContext, error: DyncordError) {
    ///     let error: Option<&MyError> = error.downcast();
    ///
    ///     // You can now use your custom error type, if the returned error is of such type.
    /// }
    /// ```
    ///
    /// Returns:
    /// * `Some(T)` - Your error, if correctly downcasted.
    /// * `None` - Nothing if the error wasn't of the type you specified.
    pub fn downcast<T>(&self) -> Option<&T>
    where
        T: Error + 'static,
    {
        match self {
            DyncordError::Command(error) => match error {
                CommandError::Arguments(error) => {
                    if let ArgumentError::Runtime(error) = error {
                        error.downcast_ref()
                    } else {
                        None
                    }
                }
                CommandError::Runtime(error) => error.downcast_ref(),
            },
            DyncordError::Event(error) => error.downcast_ref(),
            DyncordError::Error(error) => error.downcast_ref(),
        }
    }
}

/// Context passed to error handlers when an error occurs.
#[derive(Clone)]
pub struct ErrorContext<State = ()>
where
    State: StateBound,
{
    /// The event that was being handled when this error occurred.
    pub event: Event,

    /// Your bot's state.
    pub state: State,

    /// The bot's handle, used to interact with its internal state and the Discord API.
    pub handle: Handle<State>,

    /// The context the error occurred in.
    pub original: ErrorOriginalContext<State>,
}

impl<State> ErrorContext<State>
where
    State: StateBound,
{
    /// Sends a message to the current channel if the error was raised by a command.
    /// 
    /// - For prefixed commands, this is equivalent to [`PrefixedContext::send`].
    /// - For slash commands, this is equivalent to [`SlashContext::respond`].
    /// 
    /// For non-command-returned errors, this is a no-op.
    /// 
    /// Arguments:
    /// * `message` - The message to send.
    /// 
    /// Returns:
    /// * `Ok(true)` - If the message was successfully sent.
    /// * `Ok(false)` - If the message wasn't sent because the error was non-command-returned.
    /// * `Err(ErrorHandlerError::Runtime)` - If an error occurs while sending the message.
    pub async fn send(&self, message: impl Into<String>) -> Result<bool, ErrorHandlerError> {
        match &self.original {
            ErrorOriginalContext::PrefixedContext(ctx) => {
                let _ = ctx.send(message).await.map_err(ErrorHandlerError::new)?;
            }
            ErrorOriginalContext::SlashContext(ctx) => {
                ctx.respond(message).await.map_err(ErrorHandlerError::new)?;
            }
            _ => return Ok(false),
        };

        Ok(true)
    }
}

/// Wraps the original context type in an enum to pass it down to event handlers.
#[derive(Clone)]
pub enum ErrorOriginalContext<State>
where
    State: StateBound,
{
    /// The error occurred while dynamically getting the prefixes for a message event.
    PrefixesContext(Box<PrefixesContext<State>>),

    /// The error occurred when a prefixed command was called.
    PrefixedContext(Box<PrefixedContext<State>>),

    /// The error occurred when a slash command was called.
    SlashContext(Box<SlashContext<State>>),

    /// The error occurred while handling an event.
    ///
    /// These contexts have their event normalized to [`Event`] not to pass the generic up and not
    /// to fix error handlers to a single event type.
    EventContext(Box<EventContext<State, Event>>),
}

/// The result type all error handlers' results are normalized to.
pub type ErrorHandlerResult = Result<(), ErrorHandlerError>;

/// An error occurred while an error handler was being ran, or the handler chose not to handle the
/// error passed.
#[derive(Debug, thiserror::Error, Clone)]
pub enum ErrorHandlerError {
    /// Return this type when you choose not to handle the error passed to your error handler.
    #[error(
        "The error type was not handled by the current error handler. Pass it down to the next error handler."
    )]
    NotHandled,

    /// An unintentional error occurred while running the error handler.
    #[error("While handling the error, another error occurred.")]
    Error(Arc<dyn Error + Send + Sync>),
}

impl ErrorHandlerError {
    pub fn new<T>(error: T) -> Self
    where
        T: Error + Send + Sync + 'static,
    {
        Self::Error(Arc::new(error))
    }
}

/// Normalizes an error handler's return value into an [`ErrorHandlerResult`].
pub trait IntoErrorHandlerResult {
    /// Normalizes an error handler's return value into an [`ErrorHandlerResult`].
    ///
    /// Returns:
    /// [`ErrorHandlerResult`] - The normalized error handler result.
    fn into_error_handler_result(self) -> ErrorHandlerResult;
}

impl IntoErrorHandlerResult for () {
    fn into_error_handler_result(self) -> ErrorHandlerResult {
        Ok(())
    }
}

impl<T, E> IntoErrorHandlerResult for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    fn into_error_handler_result(self) -> ErrorHandlerResult {
        match self {
            Ok(_) => Ok(()),
            Err(error) => {
                // If the error returned by the error handler was an `ErrorHandlerError`, just pass
                // it down.
                if let Some(error) = (&error as &dyn Any).downcast_ref::<ErrorHandlerError>() {
                    Err(error.clone())
                } else {
                    Err(ErrorHandlerError::Error(Arc::new(error)))
                }
            }
        }
    }
}

/// A trait implemented by error handler functions.
pub trait ErrorHandler<State, Err>: Send + Sync
where
    State: StateBound,
{
    /// Runs the error handler together with the passed arguments.
    ///
    /// Arguments:
    /// * `ctx` - The [`ErrorContext`]. Check [`ErrorContext::original`] for the original context
    ///   type.
    /// * `error` - The error that occurred.
    ///
    /// Returns:
    /// * `Ok(())` - If the handling of the error succeeded.
    /// * `Err(ErrorHandlerError)` - If an error occurred while handling the error.
    fn handle(
        &self,
        ctx: ErrorContext<State>,
        error: DyncordError,
    ) -> DynFuture<'_, ErrorHandlerResult>;
}

impl<State, Func, Fut, Res> ErrorHandler<State, DyncordError> for Func
where
    State: StateBound,
    Func: Fn(ErrorContext<State>, DyncordError) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoErrorHandlerResult,
{
    fn handle(
        &self,
        ctx: ErrorContext<State>,
        error: DyncordError,
    ) -> DynFuture<'_, ErrorHandlerResult> {
        Box::pin(async move { self(ctx, error).await.into_error_handler_result() })
    }
}

/// A trait implemented by error handler wrappers that erases the specific error type being
/// handled.
pub trait ErrorHandlerWithoutType<State>: Send + Sync
where
    State: StateBound,
{
    /// Runs the error handler together with the passed arguments.
    ///
    /// Arguments:
    /// * `ctx` - The [`ErrorContext`]. Check [`ErrorContext::original`] for the original context
    ///   type.
    /// * `error` - The error that occurred.
    ///
    /// Returns:
    /// * `Ok(())` - If the handling of the error succeeded.
    /// * `Err(ErrorHandlerError)` - If an error occurred while handling the error.
    fn handle(
        &self,
        ctx: ErrorContext<State>,
        error: DyncordError,
    ) -> DynFuture<'_, ErrorHandlerResult>;
}

/// Wraps an error handler function with a phantom error type and a phantom dummy type.
pub(crate) struct ErrorHandlerWrapper<F, Error> {
    func: F,
    _error: PhantomData<Error>,
}

impl<F, Error> ErrorHandlerWrapper<F, Error> {
    pub fn new(handler: F) -> Self {
        ErrorHandlerWrapper {
            func: handler,
            _error: PhantomData,
        }
    }
}

impl<State, Func, Error> ErrorHandlerWithoutType<State> for ErrorHandlerWrapper<Func, Error>
where
    State: StateBound,
    Func: ErrorHandler<State, Error>,
    Error: Send + Sync,
{
    fn handle(
        &self,
        ctx: ErrorContext<State>,
        error: DyncordError,
    ) -> DynFuture<'_, ErrorHandlerResult> {
        self.func.handle(ctx, error)
    }
}

/// Handles an error and any errors that occur while handling such error.
///
/// The handlers are passed to this function in layers. All handlers within the first layer will be
/// called to handle the error. If none can successfully handle the error, the error is then passed
/// to be handled by all error handlers in the next layer. This repeats until either all errors can
/// be handled or no more layers remain.
///
/// If a handler itself returns an error, that error will *also* be passed to the next layers's
/// handlers. For example:
///
/// ```text
/// handler_1(error) -> Always doesn't handle error
/// handler_2(error) -> Always fails
/// handler_3(error) -> Always doesn't handle error
/// handler_4(error) -> Always succeeds
/// handler_5(error) -> Always succeeds
///
/// handle(ctx, error, [[handler_1, handler_2], [handler_3, handler_4, handler_5]]);
///
/// handler_1(error) -> Didn't handle
/// handler_2(error) -> Fails
///
/// None could handle the error within the first group, and handler_2 returned yet another error
/// that must be handled.
///
/// handler_3(handler_2_error) -> Didn't handle
/// handler_3(error) -> Didn't handle
/// handler_4(handler_2_error) -> Success!
/// handler_4(error) -> Success!
/// handler_5(handler_2_error) -> Success!
/// handler_5(error) -> Success!
/// ```
///
/// Note how `handler_4` doesn't stop `handler_5` from being called too. When a layer is being run,
/// all handlers in that layer get called for all the to-handle errors even if a handler in that
/// layer succeeds at handling the error. A handler succeeding to handle the erorr only means the
/// successfully-handled error won't be passed down to the next layer, if any.
///
/// Let's add even more handlers and layers for a more complex example.
///
/// ```text
/// handler_1(error) -> Always fails
/// handler_2(error) -> Always doesn't handle error
/// handler_3(error) -> Always succeeds only on command errors
/// handler_4(error) -> Always fails
/// handler_5(error) -> Always fails
///
/// command() -> Error
///
/// handle(ctx, error, [[handler_1, handler_2], [handler_3, handler_4], [handler_5]]);
///
/// handler_1(error) -> Fails
/// handler_2(error) -> Didn't handle
///
/// handler_3(handler_1_error) -> Didn't handle
/// handler_3(error) -> Succeess!
/// handler_4(handler_1_error) -> Fails
/// handler_4(error) -> Fails
///
/// handler_5(handler_1_error) -> Fails
/// handler_5(handler_4_error_1) -> Fails
/// handler_5(handler_4_error_2) -> Fails
/// ```
///
/// As you can see, `handler_1`'s error gets passed all the way down because no handler in the
/// middle layers handles it properly. The original error gets properly handled by `handler_3`, so
/// that error stops getting propagated. `handler_4` raises an error twice, one per call, so
/// `handler_5` ends up handling `handler_1`'s error and both of `handler_4`'s errors. `handler_5`
/// fails, but there's no more error handler layers so 6 errors go unhandled.
///
/// Arguments:
/// * `ctx` - The error handler context.
/// * `error` - The error to handle.
/// * `handlers` - The handlers to use to handle the error. Most specific error handler first.
pub(crate) async fn handle<State>(
    ctx: ErrorContext<State>,
    error: DyncordError,
    handlers: &[Vec<Arc<dyn ErrorHandlerWithoutType<State>>>],
) where
    State: StateBound,
{
    let mut to_handle = vec![error];

    for layer in handlers {
        let mut new_to_handle = vec![];

        for error in to_handle {
            let mut is_error_handled = false;

            for handler in layer {
                let result = handler.handle(ctx.clone(), error.clone()).await;

                if let Err(error) = result {
                    if let ErrorHandlerError::Error(error) = error {
                        new_to_handle.push(DyncordError::Error(error));
                    }
                } else {
                    is_error_handled = true;
                }
            }

            if !is_error_handled {
                new_to_handle.push(error);
            }
        }

        to_handle = new_to_handle;
    }
}
