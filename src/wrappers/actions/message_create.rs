//! A wrapper around sending messages.

use thiserror::Error;
use twilight_model::{channel::{Message, message::Embed}, id::{Id, marker::{ChannelMarker, MessageMarker}}};

use crate::{DynFuture, aliases::DiscordClient};

#[derive(Debug, Error)]
pub enum CreationError {
    #[error("An error occurred while sending the message: {0}")]
    Twilight(#[from] twilight_http::Error),

    #[error("An error occurred while parsing the response from the Discord API: {0}")]
    TwilightParsing(#[from] twilight_http::response::DeserializeBodyError),
}

/// A builder for sending a message.
pub struct CreateMessage {
    /// The HTTP client to use for sending the message.
    client: DiscordClient,

    /// The ID of the channel to send the message to.
    channel_id: Id<ChannelMarker>,

    /// The content of the message to send.
    content: String,

    /// The ID of the message to reply to, if any.
    replying_to: Option<Id<MessageMarker>>,

    /// The embeds to include in the message.
    embeds: Vec<Embed>,
}

impl CreateMessage {
    pub(crate) fn new(
        client: DiscordClient,
        channel_id: Id<ChannelMarker>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            client,
            channel_id,
            content: content.into(),
            replying_to: None,
            embeds: Vec::new(),
        }
    }

    /// Sets the message to reply to.
    ///
    /// Arguments:
    /// * `message_id` - The ID of the message to reply to.
    ///
    /// Returns:
    /// [`SendMessage`] - The message builder with the reply set.
    pub fn reply(mut self, message_id: Id<MessageMarker>) -> Self {
        self.replying_to = Some(message_id);
        self
    }

    /// Adds an embed to the message.
    ///
    /// Arguments:
    /// * `embed` - The embed to add to the message.
    ///
    /// Returns:
    /// [`SendMessage`] - The message builder with the embed added.
    pub fn embed(mut self, embed: impl Into<Embed>) -> Self {
        self.embeds.push(embed.into());
        self
    }

    /// Sends the message to the specified channel.
    ///
    /// Returns:
    /// * `Ok(Message)` - The message that was sent.
    /// * `Err(SendingError)` - An error that occurred while sending the message.
    async fn send(self) -> Result<Message, CreationError> {
        let mut builder = self
            .client
            .create_message(self.channel_id)
            .embeds(&self.embeds)
            .content(&self.content);

        if let Some(reply) = self.replying_to {
            builder = builder.reply(reply);
        }

        Ok(builder.await?.model().await?)
    }
}

impl IntoFuture for CreateMessage {
    type Output = Result<Message, CreationError>;
    type IntoFuture = DynFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}
