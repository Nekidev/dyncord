use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::gateway::payload::incoming::InteractionCreate;

use crate::commands::slash::SlashCommand;
use crate::handle::Handle;
use crate::state::StateBound;
use crate::wrappers::actions::interaction_respond::{
    InteractionRespondWithDeferral, InteractionRespondWithMessage,
};

#[derive(Clone)]
pub struct SlashContext<State = ()>
where
    State: StateBound,
{
    /// Your bot's state.
    pub state: State,

    /// The event that triggered this command.
    pub event: InteractionCreate,

    /// The command data sent with the event, for easier access.
    pub event_data: CommandData,

    /// The inner bot handle.
    pub handle: Handle<State>,

    /// The current command being executed.
    pub command: SlashCommand<State>,
}

impl<State> SlashContext<State>
where
    State: StateBound,
{
    /// Respond to the command with a message.
    ///
    /// Arguments:
    /// * `content` - The content of the message to respond with.
    ///
    /// Returns:
    /// [`InteractionRespondWithMessage`] - The interaction response builder. Await it to send the
    /// response.
    pub fn respond(&self, content: impl Into<String>) -> InteractionRespondWithMessage {
        InteractionRespondWithMessage::new(
            self.handle.client.clone(),
            self.event.application_id,
            self.event.id,
            self.event.token.clone(),
            content,
        )
    }

    /// Defer the command until further work is done.
    ///
    /// Returns:
    /// [`InteractionRespondWithDeferral`] - The interaction response builder. Await it to defer
    /// the response.
    pub fn defer(&self) -> InteractionRespondWithDeferral {
        InteractionRespondWithDeferral::new(
            self.handle.client.clone(),
            self.event.application_id,
            self.event.id,
            self.event.token.clone(),
        )
    }
}
