//! A handle to interact with the bot's internal state and the Discord API.
//!
//! The [`Handle`] struct provides methods to interact with the Discord API and the bot's internal
//! state (not your custom state type). Currently, it only contains a vector of commands and a
//! method to send messages. It's usually proxied to by contexts, such as in
//! [`CommandContext::send()`](crate::commands::context::CommandContext::send) where it is used to
//! send messages to the channel the command was run in.
//!
//! To send messages, you can use the [`Handle::send()`] method, which returns a [`SendMessage`]
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

use std::sync::Arc;

use thiserror::Error;
use twilight_model::channel::Message;
use twilight_model::channel::message::Embed;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, MessageMarker};

use crate::DynFuture;
use crate::aliases::DiscordClient;
use crate::commands::CommandNode;
use crate::commands::prefixes::Prefixes;
use crate::state::StateBound;
use crate::wrappers::actions::message_create::CreateMessage;

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
    /// [`CreateMessage`] - A message builder that can be used to send the message.
    pub fn send(&self, channel_id: Id<ChannelMarker>, content: impl Into<String>) -> CreateMessage {
        CreateMessage::new(self.client.clone(), channel_id, content)
    }
}
