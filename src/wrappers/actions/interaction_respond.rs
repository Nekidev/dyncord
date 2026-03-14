//! Wrappers around responding to interactions.

use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::Id;
use twilight_model::id::marker::{ApplicationMarker, InteractionMarker};

use crate::aliases::DiscordClient;
use crate::utils::DynFuture;
use crate::wrappers::TwilightError;

/// A builder for responding to an interaction with a message.
pub struct InteractionRespondWithMessage {
    client: DiscordClient,

    application_id: Id<ApplicationMarker>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: String,

    content: String,
}

impl InteractionRespondWithMessage {
    pub(crate) fn new(
        client: DiscordClient,
        application_id: Id<ApplicationMarker>,
        interaction_id: Id<InteractionMarker>,
        interaction_token: String,
        content: impl Into<String>,
    ) -> Self {
        Self {
            client,
            application_id,
            interaction_id,
            interaction_token,
            content: content.into(),
        }
    }

    async fn send(self) -> Result<(), TwilightError> {
        self.client
            .interaction(self.application_id)
            .create_response(
                self.interaction_id,
                &self.interaction_token,
                &InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(InteractionResponseData {
                        content: Some(self.content),
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
        application_id: Id<ApplicationMarker>,
        interaction_id: Id<InteractionMarker>,
        interaction_token: String,
    ) -> Self {
        Self {
            client,
            application_id,
            interaction_id,
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
