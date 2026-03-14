use std::env::consts::OS;
use std::sync::Arc;

use thiserror::Error;
use twilight_gateway::{ConfigBuilder, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client;
use twilight_model::gateway::payload::outgoing::identify::IdentifyProperties;

use crate::commands::prefixed::prefixes::Prefixes;
use crate::commands::slash::InvalidCommandError;
use crate::commands::{
    self, CommandGroupIntoCommandNode, CommandIntoCommandNode, CommandNode, prefixed, slash,
};
use crate::events::{On, OnEvent};
use crate::handle::Handle;
use crate::state::StateBound;

/// Holds all configurations related to a bot instance.
pub struct Bot<State = ()>
where
    State: StateBound,
{
    /// The list of commands the bot will route to when a message is received.
    commands: Vec<CommandNode<State>>,

    /// The list of event handlers the bot will execute when an event is received.
    events: Vec<OnEvent<State>>,

    /// The bot's state, which can be any type you want (`Send + Sync + Clone`).
    state: State,

    /// The bot's intents, which determine which events the bot will receive from the Discord API.
    intents: Intents,

    /// The shard ID to use when connecting to the Discord API.
    shard: ShardId,

    /// The bot's prefixes getter, or [`None`] if message commands are disabled.
    prefixes: Option<Arc<dyn Prefixes<State>>>,
}

impl Default for Bot<()> {
    fn default() -> Self {
        Self::new(())
    }
}

impl<State> Bot<State>
where
    State: StateBound,
{
    /// Creates a new instance of [`Bot`].
    ///
    /// Returns:
    /// [`Bot`] - A new instance of the bot.
    pub fn new(state: State) -> Bot<State> {
        Bot {
            commands: Vec::new(),
            events: Vec::new(),
            state,
            intents: Intents::empty(),
            shard: ShardId::ONE,
            prefixes: None,
        }
    }

    /// Adds a command to the bot's command list.
    ///
    /// Arguments:
    /// * `command` - The command to add to the bot's command list.
    ///
    /// Returns:
    /// [`Bot`] - The bot instance with the added command.
    pub fn command(mut self, command: impl CommandIntoCommandNode<State>) -> Self {
        self.commands.push(command.into_command_node());
        self
    }

    /// Adds a command group to the bot's command list.
    ///
    /// For example:
    /// ```rust
    /// let bot = Bot::new(())
    ///     .nest(
    ///         CommandGroup::new("admin")
    ///             .command(Command::build("kick", kick_command))
    ///             .command(Command::build("ban", ban_command))
    ///     );
    /// ```
    ///         
    ///
    /// Arguments:
    /// * `group` - The command group to add to the bot's command list.
    ///
    /// Returns:
    /// [`Bot`] - The bot instance with the added command group.
    pub fn nest(mut self, group: impl CommandGroupIntoCommandNode<State>) -> Self {
        self.commands.push(group.into_command_node());
        self
    }

    /// Adds an event handler to the bot's event handlers list.
    ///
    /// For example, to add a handler for the `MessageCreate` event, you can do:
    /// ```
    /// Bot::new(()).on_event(On::message_create(on_message));
    ///
    /// async fn on_message(ctx: EventContext<State, MessageCreate>) {
    ///     // Handle the message create event.
    /// }
    /// ```
    ///
    /// Arguments:
    /// * `handler` - The event handler to add to the bot's event handlers list.
    ///
    /// Returns:
    /// [`Bot`] - The bot instance with the added event handler.
    pub fn on_event(mut self, handler: OnEvent<State>) -> Self {
        self.events.push(handler);
        self
    }

    /// Adds intents to the bot's intents.
    ///
    /// This can be called either once per intent or once with all intents. For example:
    /// ```rust
    /// Bot::new(()).intents(Intents::GUILD_MESSAGES).intents(Intents::DIRECT_MESSAGES);
    /// // or
    /// Bot::new(()).intents(Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES);
    /// ```
    ///
    /// Arguments:
    /// * `intents` - The intents to add to the bot's intents.
    ///
    /// Returns:
    /// [`Bot`] - The bot instance with the added intents.
    pub fn intents(mut self, intents: Intents) -> Self {
        self.intents |= intents;
        self
    }

    /// Sets the prefix or prefixes the bot will use to route message commands.
    ///
    /// This can be called with a single prefix or multiple prefixes. You can also dynamically
    /// determine the prefixes based on the message or the bot's state by passing a closure or
    /// function like follows:
    ///
    /// ```
    /// async fn get_prefixes(ctx: Context, state: State) -> Vec<String> {
    ///     // Determine prefixes based on the message or state.
    ///     vec![".".to_string(), "!".to_string()]
    /// }
    ///
    /// let bot = Bot::new(state).with_prefix(get_prefixes);
    /// ```
    pub fn with_prefix(mut self, prefixes: impl Prefixes<State> + 'static) -> Self {
        self.prefixes = Some(Arc::new(prefixes));
        self
    }

    /// Returns a handle to the bot, which can be used to interact with the bot's internal state.
    ///
    /// Note: This creates a handle using the current state of the bot. If you update the bot's
    ///       state after calling this function, the handle will not reflect those updates.
    ///
    /// Arguments:
    /// * `token` - The token used to authenticate the bot with the Discord API.
    ///
    /// Returns:
    /// [`Handle`] - A handle to the bot.
    pub fn handle(&self, token: impl Into<String>) -> Handle<State> {
        let token = token.into();
        let client = Arc::new(Client::new(token));

        Handle {
            client,
            commands: Arc::new(self.commands.clone()),
            prefixes: self.prefixes.clone(),
        }
    }

    /// Runs the bot with the provided token.
    ///
    /// This function will block the current task until the bot is stopped.
    ///
    /// Arguments:
    /// * `token` - The token used to authenticate the bot with the Discord API.
    ///
    /// Returns:
    /// * `Ok(())` - If no errors were detected during execution, though if it stopped without you
    ///   wanting it to, that may also mean an error occurred.
    /// * `Err(RunningError)` - An error occurred while running or attempting to run the bot.
    pub async fn run(mut self, token: impl Into<String>) -> Result<(), RunningError> {
        let token = token.into();
        let handle = self.handle(token.clone());

        let config = ConfigBuilder::new(token.clone(), self.intents)
            .identify_properties(IdentifyProperties::new("Dyncord", "Dyncord", OS))
            .build();

        let mut gateway = Shard::with_config(self.shard, config);

        if !self.commands.is_empty() {
            let mut has_slash_commands = false;
            let mut has_prefixed_commands = false;

            if !commands::flatten_slash(&self.commands).is_empty() {
                has_slash_commands = true;
            }

            if !commands::flatten_prefixed(&self.commands).is_empty() {
                has_prefixed_commands = true;
            }

            if has_prefixed_commands {
                self = self.on_event(On::message_create(
                    prefixed::routing::route_prefixed_command,
                ));
            }

            if has_slash_commands {
                slash::validate_commands(&commands::flatten_slash(&self.commands))
                    .map_err(RunningError::InvalidSlashCommands)?;

                self = self.on_event(On::ready(slash::registration::register));
                self = self.on_event(On::interaction_create(slash::routing::route_slash_command));
            }
        }

        while let Some(Ok(event)) = gateway.next_event(EventTypeFlags::all()).await {
            for handler in &*self.events {
                if let Some(future) =
                    handler.handle(handle.clone(), self.state.clone(), event.clone())
                {
                    tokio::spawn(future);
                }
            }
        }

        Ok(())
    }
}

/// Errors that may occur while running, or attempting to run, a [`Bot`].
#[derive(Debug, Error)]
pub enum RunningError {
    #[error("One or more slash commands are invalid. {0:?}")]
    InvalidSlashCommands(Vec<InvalidCommandError>),
}
