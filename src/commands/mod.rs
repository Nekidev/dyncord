//! Dyncord prefixed and slash commands.
//!
//! This module is divided into two sub-modules, [`prefixed`] and [`slash`].
//!
//! - [`prefixed`] - Good old prefixed commands. E.g. `!help`.
//! - [`slash`] - Brand new slash commands. E.g. `/help`.
//!
//! Dyncord allows you to use both in the same bot, registration and routing is done automatically.
//!
//! # Prefixed Commands
//!
//! Prefixed commands are message-based and are executed when the user sends a prefix plus the
//! command name. For example, where the prefix is `!`, the command's name is `hello`, and it takes
//! a string argument, users can invoke the command by sending `!hello Mike` in a channel the bot
//! has access to.
//!
//! To define such commands, create an async function that takes
//! [`PrefixedContext`](prefixed::context::PrefixedContext) as its first argument. For example, to
//! build our `hello` command mentioned in the example above, the function would be
//!
//! ```
//! async fn handle_hello(ctx: PrefixedContext) {}
//! ```
//!
//! That command took one string argument, so let's make it an argument in the function too.
//!
//! ```
//! async fn handle_hello(ctx: PrefixedContext, name: String) {}
//! ```
//!
//! Now, let's register it in our bot
//!
//! ```
//! let bot = Bot::new(())
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .intents(Intents::GUILD_MESSAGES)
//!     .with_prefix("!")
//!     .command(Command::prefixed("hello", handle_hello));
//!
//! bot.run("token").await;
//! ```
//!
//! Great! Our bot now has a hello command. Try inviting the bot to your server and you'll see the
//! bot comes online when you run your binary. The command is invoked by sending `!hello yourname`.
//! However, no matter what you send, the bot does nothing. It's still correctly handling your
//! command, it just wasn't told to do anything when that happens yet. Let's do that.
//!
//! In your command handler function, add the following:
//!
//! ```
//! async fn handle_hello(ctx: PrefixedContext, name: String) {
//!     ctx.send(format!("Hey there, {name}!")).await.unwrap();
//! }
//! ```
//!
//! Try re-running your bot, then send `!hello yourname` in your server. Voilà! The bot responded
//! according to the `handle_hello` function. Well done!
//!
//! You may have realized that if you pass more than a single word to the command, it just says hi
//! back with the first word. By default, [`String`] arguments take one word as an argument.
//! However, dyncord will also take more than one word as the argument if you quote the argument.
//! Try sending `!hello "Mike Wazowski"`. It works! What about single quotes?
//! `!hello 'Mike Wazowski'` and it also works! `!hello 'Mike\'s Cat'` and woah? Guess what, it
//! also works!
//!
//! For a more in depth introduction into prefixed commands, or just as a reference, check the
//! [`prefixed` module's documentation](prefixed). It has a more in depth guide about how to use
//! prefixed commands to their full power. Good luck!
//!
//! # Slash Commands
//!
//! Slash commands are more structured API-wise than prefixed commands. This is due to Discord's
//! need to know about every command, command group, and command argument in advance. The creation
//! of slash commands is almost as simple as the creation of prefixed commands, and type-safe so
//! that if any metadata is missing, you'll get an error at compile time instead of at runtime.
//!
//! Like prefixed commands, slash commands take a first required argument called the context. It
//! contains data about the command's invocation, together with metadata about the command itself
//! and functions to respond to the interaction. Such context type is
//! [`SlashContext`](slash::context::SlashContext).
//!
//! To create your first slash command handler, create an async function that takes
//! [`SlashContext`](slash::context::SlashContext) as its first argument.
//!
//! ```
//! async fn handle_hello(ctx: SlashContext) {
//!     ctx.respond("Hey there!").await.unwrap();
//! }
//! ```
//!
//! Let's pass it to our [`Bot`](crate::Bot).
//!
//! ```
//! let bot = Bot::new(()).command(Command::slash("hello", handle_hello));
//!
//! bot.run("token").await;
//! ```
//!
//! Great. Run your binary, get on Discord, and you'll see the bot is online and when you type
//! `/hello` your new command appears. Try running it!
//!
//! Now, let's add an argument to it, because right now our command doesn't know who to say hi to.
//! Let's add a name argument.
//!
//! ```
//! async fn handle_hello(ctx: SlashContext, name: String) {
//!     ctx.respond(format!("Hey {name}!")).await.unwrap();
//! }
//! ```
//!
//! Perfect. Try to run it and now... it fails to run. Why?
//!
//! Remember that some paragraphs ago, we said Discord needs some more information when creating a
//! command than just the argument name in the handler. If dyncord were to send the command as-is,
//! Discord would fail to register it because it would be lacking metadata, and therefore dyncord
//! fails early.
//! 
//! Lets fix our error by passing metadata about our `name` argument to our [`Command`] builder.
//! Argument (options, as Discord calls them) metadata is defined through
//! [`Argument`](slash::arguments::Argument). In this case, we want to create a string argument so
//! we'll use [`Argument::string()`](slash::arguments::Argument::string).
//! 
//! ```
//! let bot = Bot::new(()).command(
//!     Command::slash("hello", handle_hello)
//!         .argument(Argument::string("name"))
//! );
//! ```
//!
//! That's it! Go back to Discord and check your command. When you type `/hello` in your message
//! bar, you'll see that it now shows one option called "name". Perfect.
//!
//! You will see that the command's summary is "A Dyncord command." and the argument's summary is
//! "A Dyncord argument." To change those defaults, both the command builder and the argument
//! builder have `.description()` associated functions you can call.
//!
//! For example,
//!
//! ```
//! let bot = Bot::new(()).command(
//!     Command::slash("hello", handle_hello)
//!         .description("Says hi to someone.")
//!         .argument(
//!             Argument::string("name")
//!                 .description("Your name, to sell it to *ahem* to say hi.")
//!         )
//! );
//! ```
//!
//! If you run your binary again, you'll see that after some seconds the descriptions get updated.
//!
//! Last but not least, there's always the need for optional arguments. However, you'll see that
//! just passing `Option<String>` to your command handler as an argument will fail to run once
//! again. You have to mark your argument as optional by calling `.optional()` on it.
//!
//! This is only a short introduction to building slash commands with Dyncord. For a more extensive
//! documentation on how to implement them, check out the [`slash` module's documentation](slash).
//! It has all the details you'll need to be able to create slash commands more in detail. For now,
//! happy coding!

use crate::commands::prefixed::{
    PrefixedCommand, PrefixedCommandBuilder, PrefixedCommandGroup, PrefixedCommandGroupBuilder,
    PrefixedCommandHandler,
};
use crate::commands::slash::{
    SlashCommand, SlashCommandBuilder, SlashCommandGroup, SlashCommandGroupBuilder,
    SlashCommandHandler,
};
use crate::state::StateBound;

pub mod prefixed;
pub mod slash;

/// Either a command or a command group.
#[derive(Clone)]
pub enum CommandNode<State>
where
    State: StateBound,
{
    PrefixedCommand(PrefixedCommand<State>),
    PrefixedCommandGroup(PrefixedCommandGroup<State>),
    SlashCommand(SlashCommand<State>),
    SlashCommandGroup(SlashCommandGroup<State>),
}

/// Converts all command types (slash, prefixed) and their builder types into [`CommandNode`]s.
pub trait CommandIntoCommandNode<State>
where
    State: StateBound,
{
    /// Converts the current type into a [`CommandNode`].
    ///
    /// Returns:
    /// [`CommandNode`] - The resulting command node.
    fn into_command_node(self) -> CommandNode<State>;
}

impl<State> CommandIntoCommandNode<State> for PrefixedCommand<State>
where
    State: StateBound,
{
    fn into_command_node(self) -> CommandNode<State> {
        CommandNode::PrefixedCommand(self)
    }
}

impl<State> CommandIntoCommandNode<State> for PrefixedCommandBuilder<State>
where
    State: StateBound,
{
    fn into_command_node(self) -> CommandNode<State> {
        CommandNode::PrefixedCommand(self.build())
    }
}

impl<State> CommandIntoCommandNode<State> for SlashCommand<State>
where
    State: StateBound,
{
    fn into_command_node(self) -> CommandNode<State> {
        CommandNode::SlashCommand(self)
    }
}

impl<State> CommandIntoCommandNode<State> for SlashCommandBuilder<State>
where
    State: StateBound,
{
    fn into_command_node(self) -> CommandNode<State> {
        CommandNode::SlashCommand(self.build())
    }
}

/// A unified API to build commands.
///
/// This type's associated functions initialize specialized command types depending on what type is
/// being initialized. This type does not represent a command directly.
pub struct Command;

impl Command {
    /// Creates a new prefixed command builder with the given name and handler.
    ///
    /// Arguments:
    /// * `name` - The command's name, used to invoke the command.
    /// * `handler` - The command's handler, the function that executes when the command is run.
    ///
    /// Returns:
    /// [`PrefixedCommandBuilder`] - A new command builder with the given name and handler.
    pub fn prefixed<State, F, Args>(
        name: impl Into<String>,
        handler: F,
    ) -> PrefixedCommandBuilder<State>
    where
        F: PrefixedCommandHandler<State, Args> + 'static,
        Args: Send + Sync + 'static,
        State: StateBound,
    {
        PrefixedCommandBuilder::new(name, handler)
    }

    /// Creates a new slash command builder with the given name and handler.
    ///
    /// Arguments:
    /// * `name` - The command's name, used to invoke the command.
    /// * `handler` - The command's handler, the function that executes when the command is run.
    ///
    /// Returns:
    /// [`SlashCommandBuilder`] - A new slash command builder with the given name.
    pub fn slash<State, F, Args>(name: impl Into<String>, handler: F) -> SlashCommandBuilder<State>
    where
        F: SlashCommandHandler<State, Args> + 'static,
        State: StateBound,
        Args: Send + Sync + 'static,
    {
        SlashCommandBuilder::new(name.into(), handler)
    }
}

/// A unified API to build command groups.
///
/// This type's associated functions initialize specialized command group types depending on what
/// type is being initialized. This type does not represent a command group directly.
pub struct CommandGroup;

impl CommandGroup {
    /// Intializes a prefixed command group builder.
    ///
    /// Arguments:
    /// * `name` - The command group's name.
    ///
    /// Returns:
    /// [`PrefixedCommandGroupBuilder`] - A new prefixed command group builder.
    pub fn prefixed<State>(name: impl Into<String>) -> PrefixedCommandGroupBuilder<State>
    where
        State: StateBound,
    {
        PrefixedCommandGroupBuilder::new(name)
    }

    /// Intializes a slash command group builder.
    ///
    /// Arguments:
    /// * `name` - The command group's name.
    ///
    /// Returns:
    /// [`SlashCommandGroupBuilder`] - A new slash command group builder.
    pub fn slash<State>(name: impl Into<String>) -> SlashCommandGroupBuilder<State>
    where
        State: StateBound,
    {
        SlashCommandGroupBuilder::new(name)
    }
}

/// Converts all command group types (slash, prefixed) and their builder types into a command node.
pub trait CommandGroupIntoCommandNode<State>
where
    State: StateBound,
{
    /// Converts the current type into a [`CommandNode`].
    ///
    /// Returns:
    /// [`CommandNode`] - The resulting command node.
    fn into_command_node(self) -> CommandNode<State>;
}

/// Flattens a [`CommandNode`] tree into a list of [`PrefixedCommand`]s.
///
/// Arguments:
/// * `nodes` - The nodes to flatten, which is a list of commands and command groups.
///
/// Returns:
/// [`Vec<PrefixedCommand>`] - A list of all the commands in the tree.
pub fn flatten_prefixed<State>(nodes: &[CommandNode<State>]) -> Vec<&PrefixedCommand<State>>
where
    State: StateBound,
{
    let mut commands = Vec::new();

    for node in nodes {
        match node {
            CommandNode::PrefixedCommand(command) => commands.push(command),
            CommandNode::PrefixedCommandGroup(group) => {
                commands.extend(flatten_prefixed(&group.children))
            }
            _ => {}
        }
    }

    commands
}

/// Flattens a [`CommandNode`] tree into a list of [`SlashCommand`]s.
///
/// Arguments:
/// * `nodes` - The nodes to flatten, which is a list of commands and command groups.
///
/// Returns:
/// [`Vec<SlashCommand>`] - A list of all the commands in the tree.
pub fn flatten_slash<State>(nodes: &[CommandNode<State>]) -> Vec<&SlashCommand<State>>
where
    State: StateBound,
{
    let mut commands = Vec::new();

    for node in nodes {
        match node {
            CommandNode::SlashCommand(command) => commands.push(command),
            CommandNode::PrefixedCommandGroup(group) => {
                commands.extend(flatten_slash(&group.children))
            }
            _ => {}
        }
    }

    commands
}

/// Returns all the prefixed commands in a list of [`CommandNode`]s.
///
/// Sub-commands inside command groups are not returned.
///
/// Arguments:
/// * `nodes` - The nodes to get the commands from, which is a list of commands and command groups.
///
/// Returns:
/// [`Vec<SlashCommand>`] - A list of all the prefixed commands in the list of nodes, excluding
/// sub-commands in command groups.
pub fn get_prefixed_commands<State>(nodes: &[CommandNode<State>]) -> Vec<&PrefixedCommand<State>>
where
    State: StateBound,
{
    let mut commands = Vec::new();

    for node in nodes {
        if let CommandNode::PrefixedCommand(command) = node {
            commands.push(command);
        }
    }

    commands
}

/// Returns all the slash commands in a list of [`CommandNode`]s.
///
/// Sub-commands inside command groups are not returned.
///
/// Arguments:
/// * `nodes` - The nodes to get the commands from, which is a list of commands and command groups.
///
/// Returns:
/// [`Vec<SlashCommand>`] - A list of all the prefixed commands in the list of nodes, excluding
/// sub-commands in command groups.
pub fn get_slash_commands<State>(nodes: &[CommandNode<State>]) -> Vec<&SlashCommand<State>>
where
    State: StateBound,
{
    let mut commands = Vec::new();

    for node in nodes {
        if let CommandNode::SlashCommand(command) = node {
            commands.push(command);
        }
    }

    commands
}

/// Returns all the prefixed command groups in a list of [`CommandNode`]s.
///
/// Sub-groups inside command groups are not returned.
///
/// Arguments:
/// * `nodes` - The nodes to get the command groups from, which is a list of commands and command
///   groups.
///
/// Returns:
/// [`Vec<PrefixedCommandGroup>`] - A list of all the command groups in the list of nodes,
/// excluding sub-groups in command groups.
pub fn get_prefixed_groups<State>(nodes: &[CommandNode<State>]) -> Vec<&PrefixedCommandGroup<State>>
where
    State: StateBound,
{
    let mut groups = Vec::new();

    for node in nodes {
        if let CommandNode::PrefixedCommandGroup(group) = node {
            groups.push(group);
        }
    }

    groups
}

/// Returns all the slash command groups in a list of [`CommandNode`]s.
///
/// Sub-groups inside command groups are not returned.
///
/// Arguments:
/// * `nodes` - The nodes to get the command groups from, which is a list of commands and command
///   groups.
///
/// Returns:
/// [`Vec<SlashCommandGroup>`] - A list of all the command groups in the list of nodes,
/// excluding sub-groups in command groups.
pub fn get_slash_groups<State>(nodes: &[CommandNode<State>]) -> Vec<&SlashCommandGroup<State>>
where
    State: StateBound,
{
    let mut groups = Vec::new();

    for node in nodes {
        if let CommandNode::SlashCommandGroup(group) = node {
            groups.push(group);
        }
    }

    groups
}
