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

use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, MessageMarker};

use crate::aliases::DiscordClient;
use crate::cache::Cache;
use crate::commands::CommandNode;
use crate::commands::prefixed::prefixes::Prefixes;
use crate::errors::ErrorHandlerWithoutType;
use crate::state::StateBound;
use crate::wrappers::TwilightError;
use crate::wrappers::actions::message_create::MessageCreate;
use crate::wrappers::types::members::Member;
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
    ///
    /// Dyncord has not yet wrapped all of this client's functions with nicer APIs. Use when
    /// dyncord lacks functionality.
    pub client: DiscordClient,

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

    #[error("The emoji passed was not a valid one.")]
    InvalidEmoji,
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

    /// Fetches a server member from the Discord API.
    ///
    /// If there's a cache backend, it's updated with this value on success.
    ///
    /// Arguments:
    /// * `server_id` - The ID of the server to fetch the member from.
    /// * `member_id` - The user ID of the member to fetch.
    ///
    /// Returns:
    /// * `Ok(Member)` - The fetched member.
    /// * `Err(HandleError)` - If an error occurred while fetching the member or saving it to
    ///   cache.
    pub async fn fetch_member(
        &self,
        server_id: u64,
        member_id: u64,
    ) -> Result<Member, HandleError> {
        let member: Member = self
            .client
            .guild_member(Id::new(server_id), Id::new(member_id))
            .await
            .map_err(TwilightError::Twilight)?
            .model()
            .await
            .map_err(TwilightError::TwilightParsing)?
            .into();

        if let Some(cache) = &self.cache {
            cache.set_member(server_id, member.clone()).await?;
        }

        Ok(member)
    }

    /// Gets a server member from the cache, or from the Discord API if not in cache.
    ///
    /// This function saves the member in cache when the Discord API is called.
    ///
    /// Arguments:
    /// * `server_id` - The ID of the server to get the member from.
    /// * `member_id` - The user ID of the member to get.
    ///
    /// Returns:
    /// * `Ok(Member)` - The server member.
    /// * `Err(HandleError)` - If an error occurred.
    pub async fn get_or_fetch_member(
        &self,
        server_id: u64,
        member_id: u64,
    ) -> Result<Member, HandleError> {
        if let Some(cache) = &self.cache {
            let member = cache.get_member_by_id(server_id, member_id).await?;

            if let Some(user) = member {
                return Ok(user);
            }
        }

        self.fetch_member(server_id, member_id).await
    }

    /// Adds a reaction to a message.
    ///
    /// For example:
    /// ```
    /// ctx.handle.add_reaction(
    ///     channel_id,
    ///     message_id,
    ///     "🍌",
    /// ).await?;
    ///
    /// ctx.handle.add_reaction(
    ///     channel_id,
    ///     message_id,
    ///     "banana",
    /// ).await?;
    ///
    /// ctx.handle.add_reaction(
    ///     channel_id,
    ///     message_id,
    ///     "<a:animated_banana:1234567890>",
    /// ).await?;
    /// ```
    ///
    /// Arguments:
    /// * `channel_id` - The ID of the channel the message to react to is in.
    /// * `message_id` - The ID of the message to react to.
    /// * `emoji` - The emoji to react with.
    ///
    /// Returns:
    /// * `Ok(())` - When the reaction is added.
    /// * `Err(HandleError)` - If an error occurred while sending the emoji.
    pub async fn add_reaction(
        &self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
        emoji: impl IntoRequestReactionType<'_>,
    ) -> Result<(), HandleError> {
        self.client
            .create_reaction(channel_id, message_id, &emoji.into_request_reaction_type()?)
            .await
            .map_err(TwilightError::from)?;

        Ok(())
    }
}

/// Trait to convert an emoji to a [`RequestReactionType`], to react to messages.
///
/// Its implementation on [`Into<String>`] lets strings like the following be parsed:
///
/// - `"🍌"`
/// - `"banana"` ([GitHub's emoji shorcodes](https://github.com/github/gemoji))
/// - `"<a:animated_banana:1234567890>"`
pub trait IntoRequestReactionType<'a> {
    /// Converts the current value into an emoji to react with.
    ///
    /// Returns:
    /// * `Ok(RequestReactionType)` - The parsed emoji to react with.
    /// * `Err(HandleError::InvalidEmoji)` - If the emoji was invalid.
    fn into_request_reaction_type(self) -> Result<RequestReactionType<'a>, HandleError>;
}

impl<'a, T> IntoRequestReactionType<'a> for T
where
    T: Into<String>,
{
    fn into_request_reaction_type(self) -> Result<RequestReactionType<'a>, HandleError> {
        let string = self.into();

        match emojis::get(&string) {
            Some(emoji) => Ok(RequestReactionType::Unicode {
                name: emoji.as_str(),
            }),
            None => match emojis::get_by_shortcode(&string) {
                Some(emoji) => Ok(RequestReactionType::Unicode {
                    name: emoji.as_str(),
                }),
                None => Ok(RequestReactionType::Custom {
                    id: string.parse().map_err(|_| HandleError::InvalidEmoji)?,
                    name: None,
                }),
            },
        }
    }
}
