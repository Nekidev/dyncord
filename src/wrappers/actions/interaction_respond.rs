//! Wrappers around responding to interactions.

use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::Id;
use twilight_model::id::marker::{ApplicationMarker, InteractionMarker};

use crate::aliases::DiscordClient;
use crate::utils::DynFuture;
use crate::wrappers::{IntoId, TwilightError};

/// A builder for responding to an interaction with a message.
pub struct InteractionRespondWithMessage {
    client: DiscordClient,

    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: String,

    content: String,

    is_ephemeral: bool,
    is_suppressing_embeds: bool,
    is_suppressing_notifications: bool,
}

impl InteractionRespondWithMessage {
    pub(crate) fn new(
        client: DiscordClient,
        application_id: impl IntoId<ApplicationMarker>,
        interaction_id: impl IntoId<InteractionMarker>,
        interaction_token: String,
        content: impl Into<String>,
    ) -> Self {
        Self {
            client,
            application_id: application_id.into_id(),
            interaction_id: interaction_id.into_id(),
            interaction_token,
            is_ephemeral: false,
            is_suppressing_embeds: false,
            is_suppressing_notifications: false,
            content: content.into(),
        }
    }

    /// Makes the sent message only be visible to the user that triggered the interaction.
    /// 
    /// Returns:
    /// [`InteractionRespondWithMessage`] - The current response builder with ephemerality set.
    pub fn ephemeral(mut self) -> Self {
        self.is_ephemeral = true;
        self
    }

    /// Prevent any embeds from showing to users.
    /// 
    /// Returns:
    /// [`InteractionRespondWithMessage`] - The current response builder with embeds blocked.
    pub fn no_embeds(mut self) -> Self {
        self.is_suppressing_embeds = true;
        self
    }

    /// Prevent desktop and push notifications from being sent to users about this message.
    /// 
    /// Returns:
    /// [`InteractionRespondWithMessage`] - The current response builder with notifications
    /// blocked.
    pub fn no_notifications(mut self) -> Self {
        self.is_suppressing_notifications = true;
        self
    }

    async fn send(self) -> Result<(), TwilightError> {
        let mut flags = MessageFlags::empty();

        if self.is_ephemeral {
            flags |= MessageFlags::EPHEMERAL;
        }

        if self.is_suppressing_embeds {
            flags |= MessageFlags::SUPPRESS_EMBEDS;
        }

        if self.is_suppressing_notifications {
            flags |= MessageFlags::SUPPRESS_NOTIFICATIONS;
        }

        self.client
            .interaction(self.application_id)
            .create_response(
                self.interaction_id,
                &self.interaction_token,
                &InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(InteractionResponseData {
                        content: Some(self.content),
                        flags: Some(flags),
                        ..Default::default()
                    }),
                },
            )
            .await?;

        Ok(())
    }
}

impl IntoFuture for InteractionRespondWithMessage {
    type Output = Result<(), TwilightError>;
    type IntoFuture = DynFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}

/// A builder for responding to an interaction with deferral.
pub struct InteractionRespondWithDeferral {
    client: DiscordClient,

    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: String,
}

impl InteractionRespondWithDeferral {
    pub(crate) fn new(
        client: DiscordClient,
        application_id: impl IntoId<ApplicationMarker>,
        interaction_id: impl IntoId<InteractionMarker>,
        interaction_token: String,
    ) -> Self {
        Self {
            client,
            application_id: application_id.into_id(),
            interaction_id: interaction_id.into_id(),
            interaction_token,
        }
    }

    async fn send(self) -> Result<(), TwilightError> {
        self.client
            .interaction(self.application_id)
            .create_response(
                self.interaction_id,
                &self.interaction_token,
                &InteractionResponse {
                    kind: InteractionResponseType::DeferredChannelMessageWithSource,
                    data: None,
                },
            )
            .await?;

        Ok(())
    }
}

impl IntoFuture for InteractionRespondWithDeferral {
    type Output = Result<(), TwilightError>;
    type IntoFuture = DynFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}
