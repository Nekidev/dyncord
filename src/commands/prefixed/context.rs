use twilight_model::gateway::payload::incoming::MessageCreate as TwilightMessageCreate;

use crate::handle::Handle;
use crate::state::StateBound;
use crate::wrappers::actions::message_create::MessageCreate;

#[derive(Clone)]
pub struct PrefixedContext<State = ()>
where
    State: StateBound,
{
    /// Your bot's state.
    pub state: State,

    /// The event that triggered the execution of this command.
    pub event: TwilightMessageCreate,

    /// The internal handle, used to interact with the [`Bot`](crate::Bot) and the Discord API.
    pub handle: Handle<State>,

    /// The name used to invoke this command.
    pub command_identifier: String,

    /// The prefix used to invoke this command.
    pub command_prefix: String,

    /// The raw arguments passed to the command, without the prefix and command name.
    pub command_args: String,
}

impl<State> PrefixedContext<State>
where
    State: StateBound,
{
    /// Sends a message in the channel the command was run.
    ///
    /// Arguments:
    /// * `content` - The content of the message to send.
    ///
    /// Returns:
    /// [`MessageCreate`] - A message builder that is awaited to send the message.
    pub fn send(&self, content: impl Into<String>) -> MessageCreate {
        self.handle.send(self.event.channel_id, content.into())
    }

    /// Replies to the message that triggered the command.
    ///
    /// Note: This will not work if the command was triggered by a message that is too old to reply
    ///       to.
    ///
    /// Arguments:
    /// * `content` - The content of the message to send.
    ///
    /// Returns:
    /// [`MessageCreate`] - A message builder that is awaited to send the message.
    pub fn reply(&self, content: impl Into<String>) -> MessageCreate {
        self.handle
            .send(self.event.channel_id, content.into())
            .reply(self.event.id)
    }
}
