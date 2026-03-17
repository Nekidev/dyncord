//! Command permission checkers.
//!
//! Not all commands are meant to be run by all users, nor in all contexts. For example, moderation
//! commands must only be run by moderators. Dyncord supports permission checking before a command
//! is run, which simplifies the task of checking if the user is allowed to run the command.
//!
//! # Quick Start
//!
//! > Note: This example uses prefixed commands, but the same code is valid for slash commands.
//! >       Permission checks are command type-indifferent.
//!
//! Let's simulate we're making a gardening bot. Our bot will be simple, it'll have a `water`
//! command to water our plants. Plants must not be watered at night, though, since it attracts
//! pests. Let's start by creating our command.
//!
//! ```
//! async fn handle_water(ctx: PrefixedContext) {
//!     ctx.send("Watering plants...").await.ok();
//! }
//!
//! let bot = Bot::new(())
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .with_prefix("!")
//!     .command(Command::prefixed("water", handle_water));
//!
//! bot.run("token").await.unwrap();
//! ```
//!
//! Perfect. Now when we run `!water`, our plants get watered.
//!
//! All permission checkers follow the same signature:
//!
//! ```
//! async fn function_name(ctx: PermissionContext) -> Result<(), Error>;
//! ```
//!
//! You can set `Error` to any error type you want. Handling is simple: `Ok(())` means the user is
//! allowed to run the command, `Err(Anything)` means the user is not allowed to run the command.
//!
//! Let's create our permission checker and a custom error for when it's not daytime.
//!
//! ```
//! #[derive(Debug, thiserror::Error)]
//! #[error("It's not daytime! Water your plants tomorrow.")]
//! struct NotDaytime;
//!
//! async fn is_daytime(ctx: PermissionContext) -> Result<(), NotDaytime> {
//!     Ok(())
//! }
//! ```
//!
//! We'll use [`chrono`] to check whether it's daytime.
//!
//! ```
//! async fn is_daytime(ctx: PermissionContext) -> Result<(), NotDaytime> {
//!     let now = Local::now();
//!
//!     if now.hour() < 8 || now.hour() >= 20 {
//!         return Err(NotDaytime);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! Now, let's add our permission checker to our command.
//!
//! ```
//! let bot = Bot::new(())
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .with_prefix("!")
//!     .command(Command::prefixed("water", handle_water).check(is_daytime));  // This line.
//! ```
//!
//! Great! Try running your command. You'll see that if it's not between 8 AM and 8 PM the bot
//! won't respond. Good.
//!
//! Now, it's not really user-friendly if we just go silent when it's not daytime. We have to let
//! the user know why the command is not being run. Let's make an error handler for that.
//!
//! > Note: If you're not familiar with error handling, you may want to read
//! > [the error-handling documentation](crate::errors).
//!
//! ```
//! async fn on_error(ctx: ErrorContext, error: DyncordError) {
//!     // We check if the inner error is our NotDaytime error type.
//!     if error.downcast::<NotDaytime>().is_some() {
//!         ctx.send("Don't water plants at night! Come back once there's daylight.").await.ok();
//!     }
//! }
//! ```
//!
//! We're just missing to add our error handler to our bot.
//!
//! ```
//! let bot = Bot::new(())
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .with_prefix("!")
//!     .command(Command::prefixed("water", handle_water).check(is_daytime))
//!     .on_error(on_error);  // Add this line.
//! ```
//!
//! Re-run your bot and try running your command. If it's daytime, the command will run normally.
//! However, if it's past 8 PM and before 8 AM, you'll instead see the error message. Well done!
//!
//! # Multiple Error Handlers
//!
//! It's common to want to make multiple checks before running a command. With dyncord, you can add
//! multiple checks to a command and they'll be checked for in order before the command is run. If
//! any of those checks fails, the command is not run and that error is passed to the corresponding
//! error handler.
//!
//! For example:
//!
//! ```
//! Command::prefixed("hello", handle_hello)
//!     .check(my_check_1)
//!     .check(my_check_2)
//!     .check(my_check_3);
//! ```
//!
//! They'll run serially in order before the command is run.

use std::error::Error;
use std::sync::Arc;

use twilight_gateway::Event;

use crate::handle::Handle;
use crate::state::StateBound;
use crate::utils::DynFuture;
use crate::wrappers::types::users::User;

/// The context in which the permission is being checked.
#[derive(Clone)]
pub struct PermissionContext<State = ()>
where
    State: StateBound,
{
    /// Your custom bot state.
    pub state: State,

    /// The bot's handle, used to interact with its internal state and the Discord API.
    pub handle: Handle<State>,

    /// The event that triggered this permission check.
    pub event: Event,
}

impl<State> PermissionContext<State>
where
    State: StateBound,
{
    /// Returns the user who run the command, if Discord sent their data.
    ///
    /// Returns:
    /// * `Ok(User)` - If Discord sent the data of the user who run the command.
    /// * `None` - If Discord didn't send the data of the user who run the command.
    pub fn user(&self) -> Option<User> {
        match &self.event {
            Event::MessageCreate(event) => Some(event.author.clone().into()),
            Event::InteractionCreate(event) => {
                match (&event.user, &event.member) {
                    (Some(user), None) => Some(user.clone().into()),
                    (None, Some(member)) => {
                        member.user.as_ref().map(|user| user.clone().into())
                    }
                    _ => {
                        // Discord sent neither a user nor a member.
                        None
                    }
                }
            }
            _ => unreachable!("Permissions can't be checked for outside command contexts!"),
        }
    }

    /// Returns the ID of the channel the command was run in, if sent by Discord.
    ///
    /// Returns:
    /// * `Ok(User)` - If Discord sent the data of the channel in which the command was run.
    /// * `None` - If Discord didn't send the data of the channel in which the command was run.
    pub fn channel_id(&self) -> Option<u64> {
        match &self.event {
            Event::MessageCreate(event) => Some(event.channel_id.get()),
            Event::InteractionCreate(event) => event.channel.as_ref().map(|c| c.id.get()),
            _ => {
                unreachable!("Permissions can't be checked for outside command contexts!");
            }
        }
    }

    /// Returns the ID of the server the command was run in, if sent by Discord.
    ///
    /// Returns:
    /// * `Ok(User)` - If Discord sent the data of the server in which the command was run.
    /// * `None` - If Discord didn't send the data of the server in which the command was run.
    pub fn server_id(&self) -> Option<u64> {
        match &self.event {
            Event::MessageCreate(event) => event.guild_id.as_ref().map(|i| i.get()),
            Event::InteractionCreate(event) => event.guild_id.as_ref().map(|i| i.get()),
            _ => {
                unreachable!("Permissions can't be checked for outside command contexts!");
            }
        }
    }
}

/// An error returned by a permission checker.
pub type PermissionError = Arc<dyn Error + Send + Sync>;

/// The result type all permission checksers must return.
pub type PermissionResult = Result<(), PermissionError>;

/// Normalizes permission checker results into [`PermissionResult`].
pub trait IntoPermissionResult {
    /// Normalizes permission checker results into [`PermissionResult`].
    ///
    /// Returns:
    /// [`PermissionResult`] - The permission checker's result.
    fn into_permission_result(self) -> PermissionResult;
}

impl<Err> IntoPermissionResult for Result<(), Err>
where
    Err: Error + Send + Sync + 'static,
{
    fn into_permission_result(self) -> PermissionResult {
        match self {
            Ok(_) => Ok(()),
            Err(error) => Err(Arc::new(error)),
        }
    }
}

/// A trait implemented by all permission checker functions.
pub trait PermissionChecker<State>: Send + Sync
where
    State: StateBound,
{
    /// Runs the permission check.
    ///
    /// Arguments:
    /// * `ctx` - The context in which the permission is being checked for.
    ///
    /// Returns:
    /// * `Ok(())` - If the user is allowed to run the command.
    /// * `Err(Error)` - If the user is not allowed to run the command. If the permission checker
    ///   has an unintended error, it will also be returned here. Errors returned must all be
    ///   passed down to error handlers.
    fn check(&self, ctx: PermissionContext<State>) -> DynFuture<'_, PermissionResult>;
}

impl<State, Func, Fut, Res> PermissionChecker<State> for Func
where
    State: StateBound,
    Func: Fn(PermissionContext<State>) -> Fut + Send + Sync,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoPermissionResult,
{
    fn check(&self, ctx: PermissionContext<State>) -> DynFuture<'_, PermissionResult> {
        Box::pin(async move { self(ctx).await.into_permission_result() })
    }
}
