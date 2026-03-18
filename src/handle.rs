//! A handle to interact with the bot's internal state and the Discord API.
//!
//! The [`Handle`] struct provides methods to interact with the Discord API and the bot's internal
//! state (not your custom state type). Currently, it only contains a vector of commands and a
//! method to send messages. It's usually proxied to by contexts, such as in
//! [`PrefixedContext::send()`](crate::commands::prefixed::context::PrefixedContext::send) where it
//! is used to send messages to the channel the command was run in.
//!
//! To send messages, you can use the [`Handle::send()`] method, which returns a [`MessageCreate`]
//! builder that is awaited to send the message. For example, in a command function:
//!
//! ```
//! async fn ping(ctx: CommandContext) {
//!     ctx.handle.send(ctx.event.channel_id, "Pong!").await.unwrap();
//! }
//! ```
//!
//! This is also useful, for example, to build a help command by listing the bot's commands via
//! [`Handle::commands`].
//!
//! # Cache-Backed Associated Functions
//!
//! Some functions have two variants, `fetch_*` and `get_or_fetch_*` (for example,
//! [`Handle::fetch_user`] and [`Handle::get_or_fetch_user`]). In those cases, `fetch_*` means
//! "get from the Discord API" while `get_or_fetch_*` means "get from cache, or from the Discord
//! API if not".
//!
//! When there's no cache backend, they're both effectively the same function.

use std::error::Error;
use std::sync::Arc;

use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;

use crate::aliases::DiscordClient;
use crate::cache::Cache;
use crate::commands::CommandNode;
use crate::commands::prefixed::prefixes::Prefixes;
use crate::errors::ErrorHandlerWithoutType;
use crate::state::StateBound;
use crate::wrappers::TwilightError;
use crate::wrappers::actions::message_create::MessageCreate;
use crate::wrappers::types::users::User;

/// A handle to interact with the bot's internal state and the Discord API.
///
/// Read the [module-level documentation](self) for more details and examples on how to use this.
#[derive(Clone)]
pub struct Handle<State>
where
    State: StateBound,
{
    /// The HTTP client to use for sending messages and other interactions with the Discord API.
    pub(crate) client: DiscordClient,

    /// The bot's commands.
    pub commands: Arc<Vec<CommandNode<State>>>,

    /// The prefixes getter for the bot, if any.
    pub(crate) prefixes: Option<Arc<dyn Prefixes<State>>>,

    /// The top-level error handler.
    pub(crate) on_errors: Vec<Arc<dyn ErrorHandlerWithoutType<State>>>,

    /// The cache in use, if any.
    pub cache: Option<Arc<dyn Cache>>,
}

#[derive(Debug, thiserror::Error)]
pub enum HandleError {
    #[error("An error occurred while calling the Discord API: {0}")]
    Twilight(#[from] TwilightError),

    #[error("The cache backend returned an error: {0}")]
    Cache(#[from] Box<dyn Error + Send + Sync>),
}

impl<State> Handle<State>
where
    State: StateBound,
{
    /// Sends a message to a channel.
    ///
    /// Arguments:
    /// * `channel_id` - The ID of the channel to send the message to.
    /// * `content` - The content of the message to send.
    ///
    /// Returns:
    /// [`MessageCreate`] - A message builder that can be used to send the message.
    pub fn send(&self, channel_id: Id<ChannelMarker>, content: impl Into<String>) -> MessageCreate {
        MessageCreate::new(self.client.clone(), channel_id, content)
    }

    /// Fetches a user from the Discord API.
    ///
    /// If there's a cache backend, it's updated with this value on success.
    ///
    /// Arguments:
    /// * `user_id` - The ID of the user to fetch.
    ///
    /// Returns:
    /// * `Ok(User)` - The fetched user.
    /// * `Err(HandleError)` - If an error occurred while fetching the user or saving it to cache.
    pub async fn fetch_user(&self, user_id: u64) -> Result<User, HandleError> {
        let user: User = self
            .client
            .user(Id::new(user_id))
            .await
            .map_err(TwilightError::Twilight)?
            .model()
            .await
            .map_err(TwilightError::TwilightParsing)?
            .into();

        if let Some(cache) = &self.cache {
            cache.set_user(user.clone()).await?;
        }

        Ok(user)
    }

    /// Gets a user from the cache, or from the Discord API if not in cache.
    ///
    /// This function saves the user in cache when the Discord API is called.
    ///
    /// Arguments:
    /// * `user_id` - The ID of the user to get.
    ///
    /// Returns:
    /// * `Ok(User)` - The user.
    /// * `Err(HandleError)` - If an error occurred.
    pub async fn get_or_fetch_user(&self, user_id: u64) -> Result<User, HandleError> {
        if let Some(cache) = &self.cache {
            let user = cache.get_user_by_id(user_id).await?;

            if let Some(user) = user {
                return Ok(user);
            }
        }

        self.fetch_user(user_id).await
    }
}
