//! Slash commands for [`Bot`](crate::Bot).
//!
//! Slash commands are Discord's built-in command system. It replaces the classic
//! [prefix-based commands](crate::commands::prefixed) with interactive commands starting with `/`
//! and with a more self-explanatory way of using it.
//!
//! Slash commands in dyncord are just a handler function taking [`SlashContext`] as its first
//! argument, and an arbitrary amount of arguments (currently up to 6). Dyncord takes care of
//! registering your commands and of calling your handler functions when your commands are used.
//!
//! A basic slash command handler looks like follows:
//!
//! ```
//! async fn handle_command(ctx: SlashContext) {}
//! ```
//!
//! To register it, add it to your [`Bot`](crate::Bot) instance like follows:
//!
//! ```
//! let bot = Bot::new(())
//!     .command(Command::slash("command", handle_command));
//! ```
//!
//! That's all you gotta do. Try running your bot with [`Bot::run()`](crate::Bot::run) and see how
//! your bot automatically comes online and registers a new command called `/command`. Great!
//!
//! You will quickly notice, however, that when calling such command it fails. Slash commands don't
//! only need to be run when they're called, they also need to respond to the command call for
//! Discord not to show an error message. Let's fix our error by responding to the command.
//!
//! ```
//! async fn handle_command(ctx: SlashContext) {
//!     ctx.respond("Hey there, fella!").await.unwrap();
//! }
//! ```
//!
//! After restarting your bot, try calling `/command` again. Voila!
//!
//! # Arguments
//!
//! What we just did is great for basic commands, but you'll soon find yourself needing your users
//! to pass arguments to such commands. Accepting arguments is quite simple in dyncord.
//!
//! Let's make a new command that says hi back.
//!
//! ```
//! async fn handle_hello(ctx: SlashContext, name: String) {
//!     ctx.respond(format!("Hey there, {name}!")).await.unwrap();
//! }
//! ```
//!
//! Unlike prefix-based commands, where that's all you have to do to get it working, slash commands
//! also require you to register such arguments together with your command. Let's tell our
//! [`Bot`](crate::Bot) what our new argument is called and what its type is.
//!
//! ```
//! let bot = Bot::new(())
//!     .command(Command::slash("command", handle_command))
//!     .command(
//!         Command::slash("hello", handle_hello)
//!             .argument(Argument::string("name").description("Your name"))
//!     );
//! ```
//!
//! Each [`.argument()`](SlashCommandBuilder::argument) call takes an
//! [`Argument`](arguments::Argument) builder, in the order you defined them in your handler. In
//! this case, our first argument is `name` so the first argument we pass to [`Command`] is
//! `Argument::string("name")`.
//!
//! Try restarting your bot now, you'll see the new command with an argument appears when you
//! search for `/hello`. If it doesn't, give it a minute. Discord takes some time to reload them
//! sometimes. When you run the command, you'll see it responds properly to the user. Well done!
//!
//! Sometimes, though, you want to take *optional* arguments. Not always does the user need to pass
//! every single one, after all. Let's make our `name` argument optional.
//!
//! ```
//! async fn handle_hello(ctx: SlashContext, name: Option<String>) {
//!     let name = name.unwrap_or("pivacy-conscious user".into());
//!
//!     ctx.respond(format!("Hey there, {name}!")).await.unwrap();
//! }
//! ```
//!
//! If you try running the bot as-is, you'll notice it fails to start. This is because we didn't
//! mark our argument as optional when registering it on the bot. Let's fix that.
//!
//! ```
//! let bot = Bot::new(())
//!     .command(Command::slash("command", handle_command))
//!     .command(
//!         Command::slash("hello", handle_hello)
//!             .argument(
//!                 Argument::string("name")
//!                     .description("Your name")
//!                     .optional()  // Add this
//!             )
//!     );
//! ```
//!
//! Now yes. Try running your bot and see the changes. You're doing great.
//!
//! ## Primitive Argument Types
//!
//! Given this is work-in-progress support for slash commands, we only support a few argument types
//! at the moment:
//!
//! - `String` - `Argument::string()`
//! - `Option<String>` - `Argument::string().optional()`
//!
//! Argument types are those types that implement [`IntoArgument`] (not to be confused with
//! prefixed commands' [`IntoArgument`](crate::commands::prefixed::arguments::IntoArgument)).
//!
//! ## Custom Argument Types
//!
//! You can also create [`IntoArgument`] implementations for your own custom types. For example,
//! you could make a `Name(String, String)` type holding someone's first and last names. Such
//! implementation would look like follows:
//!
//! ```
//! struct Name(String, String);
//!
//! impl IntoArgument<()> for Name {
//!     // Takes the slash context and the argument data sent by discord, if any, and returns
//!     // either your custom argument or an error.
//!     //
//!     // The return type is `DynFuture` because the future needs to be able to be
//!     // `Send + 'static` due to parent type limitations. You can treat this function as either
//!     // a sync function and wrap return types in `pinbox()`, or treat it like an async function
//!     // by returning one `Box::pin(async move { ... })` that does your async work and returns
//!     // `Self`.
//!     fn into_argument_primitive(
//!         _ctx: SlashContext<()>,
//!         argument: Option<CommandDataOption>,
//!     ) -> DynFuture<'static, Result<Self, ArgumentError>> {
//!         if let Some(argument) = argument {
//!             if let CommandOptionValue::String(argument) = argument.value {
//!                 match argument.split_once(' ') {
//!                     Some((first, last)) => pinbox(Ok(Name(first.into(), last.into()))),
//!                     None => pinbox(Err(ArgumentError::InvalidValue)),
//!                 }
//!             } else {
//!                 pinbox(Err(ArgumentError::IncorrectType))
//!             }
//!         } else {
//!             pinbox(Err(ArgumentError::Missing))
//!         }
//!     }
//!
//!     // The discord-native type your custom type will be registered as.
//!     fn r#type() -> ArgumentType {
//!         ArgumentType::String
//!     }
//! }
//! ```
//! 
//! # Command Context
//! 
//! The command context is a struct that contains information about the context in which the
//! command is being executed. This includes things like the event that triggered the command, the
//! user who used it, channel, guild, your bot’s state, and more. It also provides some utility
//! functions to interact with the Discord API, such as responding with a message and deferring the
//! command's response.
//! 
//! The [`SlashContext<State>`] is the required first argument of all slash command handlers. It
//! takes a generic `State` type which is the same as the one used in your bot, or `()` by default
//! when you don’t need any state.
//! 
//! ## Responding to Command Calls
//! 
//! Without responding to a slash command's calls, Discord will display an error message to the
//! user, even if no error actually occurred.
//! 
//! To respond to a command call, there's currently two methods:
//! 
//! - [`SlashContext::respond`] - Respond with a message.
//! - [`SlashContext::defer`] - Display a loading message while you do some longer work. Call
//!   [`SlashContext::respond`] once you're done with such work.
//! 
//! For example,
//! 
//! ```
//! async fn handle_api_call(ctx: SlashCommand) -> Result<(), TwilightError> {
//!     ctx.defer().await?;
//! 
//!     match library::get_message().await {
//!         Ok(message) => ctx.respond(message).await?,
//!         Err(error) => ctx.respond(format!("An error occurred! {error:?}")).await?,
//!     };
//! 
//!     Ok(())
//! }
//! ```

pub mod arguments;
pub mod context;
pub(crate) mod registration;
pub(crate) mod routing;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use thiserror::Error;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::id::Id;

use crate::commands::slash::arguments::{ArgumentError, ArgumentMeta, ArgumentType, IntoArgument};
use crate::commands::slash::context::SlashContext;
use crate::commands::{CommandGroupIntoCommandNode, CommandNode};
use crate::state::StateBound;
use crate::utils::DynFuture;

/// A slash command.
#[derive(Clone)]
pub struct SlashCommand<State>
where
    State: StateBound,
{
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    handler: Arc<dyn SlashCommandHandlerWithoutArgs<State>>,

    arguments: Vec<ArgumentMeta>,
}

impl<State> From<SlashCommand<State>> for Command
where
    State: StateBound,
{
    fn from(value: SlashCommand<State>) -> Self {
        #[allow(deprecated)]
        Command {
            application_id: None,
            contexts: None,
            default_member_permissions: None,
            description: value.description,
            description_localizations: Some(value.description_i18n),
            guild_id: None,
            id: None,
            integration_types: None,
            kind: CommandType::ChatInput,
            name: value.name,
            name_localizations: Some(value.name_i18n),
            nsfw: None,
            options: value.arguments.into_iter().map(|arg| arg.into()).collect(),
            version: Id::new(1),
            dm_permission: None,
        }
    }
}

/// A builder for slash commands that allows setting optional extra metadata.
pub struct SlashCommandBuilder<State>
where
    State: StateBound,
{
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    handler: Arc<dyn SlashCommandHandlerWithoutArgs<State>>,

    arguments: Vec<ArgumentMeta>,
}

impl<State> SlashCommandBuilder<State>
where
    State: StateBound,
{
    pub(crate) fn new<Args>(
        name: String,
        handler: impl SlashCommandHandler<State, Args> + 'static,
    ) -> Self
    where
        Args: Send + Sync + 'static,
    {
        SlashCommandBuilder {
            name,
            name_i18n: HashMap::new(),
            description: String::from("A Dyncord command."),
            description_i18n: HashMap::new(),
            handler: Arc::new(SlashCommandHandlerWrapper::new(handler)),
            arguments: vec![],
        }
    }

    /// Sets the default description of the command.
    /// 
    /// Arguments:
    /// * `description` - The command's default description.
    /// 
    /// Returns:
    /// [`SlashCommandBuilder`] - Self with the description set.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds an argument's metadata of the command.
    /// 
    /// This function must be called on the builder in the same order the arguments are defined in
    /// the handler function. For example:
    /// 
    /// ```
    /// async fn handle(ctx: SlashContext, name: String, age: i32) -> {}
    /// 
    /// // Correct way, arguments follow the same order as in `handle`.
    /// let command = Command::slash("command", handle)
    ///     .argument(Argument::string("name"))
    ///     .argument(Argument::number("age"));
    /// 
    /// // Incorrect way, arguments are not in the same order as in `handle`.
    /// let command = Command::slash("command", handle)
    ///     .argument(Argument::number("age"))
    ///     .argument(Argument::string("name"));
    /// ```
    /// 
    /// Failing to call this function in order will cause either the bot not to run or to run with
    /// arguments whose metadata is interchanged, misguiding the user.
    /// 
    /// Arguments:
    /// * `argument` - The argument's metadata.
    /// 
    /// Returns:
    /// [`SlashCommandBuilder`] - Self with the argument metadata set.
    pub fn argument(mut self, argument: impl Into<ArgumentMeta>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    pub(crate) fn build(self) -> SlashCommand<State> {
        SlashCommand {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            handler: self.handler,
            arguments: self.arguments,
        }
    }
}

pub type CommandResult = Result<(), SlashCommandError>;

/// An error occurred while executing, or attempting to execute, a slash command.
#[derive(Debug, Error)]
pub enum SlashCommandError {
    #[error(
        "The arguments received from Discord could not be parsed into the handler's arguments: {0}"
    )]
    InvalidArgument(#[from] ArgumentError),
}

/// Trait implemented by slash command handlers.
pub trait SlashCommandHandler<State, Args>: Send + Sync
where
    State: StateBound,
{
    /// Runs the command handler.
    ///
    /// Arguments:
    /// * `ctx` - The context of the command, which contains information about the interaction,
    ///   channel, guild, etc.
    ///
    /// Returns:
    /// [`CommandResult`] - A future that resolves when the command handler has finished executing.
    /// This is equivalent to an asynchronous function, so you can just run `.run().await`.
    fn run(&self, ctx: SlashContext<State>) -> DynFuture<'_, CommandResult>;

    /// Returns the arguments the command handler actually takes.
    ///
    /// This is used to alert the developer when they haven't defined the argument metadata for it.
    ///
    /// Returns:
    /// [`Vec<ArgumentType>`] - A vector of command argument types, one per argument the function
    /// takes.
    fn argument_types(&self) -> Vec<ArgumentType>;
}

impl<State, Func, Fut, Res> SlashCommandHandler<State, ()> for Func
where
    State: StateBound,
    Func: Fn(SlashContext<State>) -> Fut + Send + Sync,
    Fut: Future<Output = Res> + Send,
{
    fn run(&self, ctx: SlashContext<State>) -> DynFuture<'_, CommandResult> {
        Box::pin(async move {
            self(ctx).await;

            Ok(())
        })
    }

    fn argument_types(&self) -> Vec<ArgumentType> {
        vec![]
    }
}

/// Parses an argument into the argument type required by the slash command handler.
/// 
/// Arguments:
/// * `ctx` - The slash command context.
/// * `options` - All the options received from Discord for this command call.
/// * `index` - The index of the argument in `ctx.command.arguments` being parsed.
/// 
/// Returns:
/// * `Ok(T)` - When the argument was correctly parsed.
/// * `Err(SlashCommandError::InvalidArgument)` - When the argument failed to parse.
async fn parse_arg<T, State>(
    ctx: SlashContext<State>,
    options: &[CommandDataOption],
    index: usize,
) -> Result<T, SlashCommandError>
where
    T: IntoArgument<State>,
    State: StateBound,
{
    let argument = ctx
        .command
        .arguments
        .get(index)
        .ok_or(ArgumentError::MissingMeta)?;
    let option = options.iter().find(|i| i.name == *argument.name());

    Ok(T::into_argument_primitive(ctx, option.cloned()).await?)
}

impl<State, Func, Fut, Res, A> SlashCommandHandler<State, (A,)> for Func
where
    State: StateBound,
    Func: Fn(SlashContext<State>, A) -> Fut + Send + Sync,
    Fut: Future<Output = Res> + Send,
    A: IntoArgument<State>,
{
    fn run(&self, ctx: SlashContext<State>) -> DynFuture<'_, CommandResult> {
        Box::pin(async move {
            let options = ctx.event_data.options.clone();

            let a = parse_arg(ctx.clone(), &options, 0).await?;

            self(ctx, a).await;

            Ok(())
        })
    }

    fn argument_types(&self) -> Vec<ArgumentType> {
        vec![A::r#type()]
    }
}

impl<State, Func, Fut, Res, A, B> SlashCommandHandler<State, (A, B)> for Func
where
    State: StateBound,
    Func: Fn(SlashContext<State>, A, B) -> Fut + Send + Sync,
    Fut: Future<Output = Res> + Send,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
{
    fn run(&self, ctx: SlashContext<State>) -> DynFuture<'_, CommandResult> {
        Box::pin(async move {
            let options = ctx.event_data.options.clone();

            let a = parse_arg(ctx.clone(), &options, 0).await?;
            let b = parse_arg(ctx.clone(), &options, 1).await?;

            self(ctx, a, b).await;

            Ok(())
        })
    }

    fn argument_types(&self) -> Vec<ArgumentType> {
        vec![A::r#type(), B::r#type()]
    }
}

impl<State, Func, Fut, Res, A, B, C> SlashCommandHandler<State, (A, B, C)> for Func
where
    State: StateBound,
    Func: Fn(SlashContext<State>, A, B, C) -> Fut + Send + Sync,
    Fut: Future<Output = Res> + Send,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
{
    fn run(&self, ctx: SlashContext<State>) -> DynFuture<'_, CommandResult> {
        Box::pin(async move {
            let options = ctx.event_data.options.clone();

            let a = parse_arg(ctx.clone(), &options, 0).await?;
            let b = parse_arg(ctx.clone(), &options, 1).await?;
            let c = parse_arg(ctx.clone(), &options, 2).await?;

            self(ctx, a, b, c).await;

            Ok(())
        })
    }

    fn argument_types(&self) -> Vec<ArgumentType> {
        vec![A::r#type(), B::r#type(), C::r#type()]
    }
}

impl<State, Func, Fut, Res, A, B, C, D> SlashCommandHandler<State, (A, B, C, D)> for Func
where
    State: StateBound,
    Func: Fn(SlashContext<State>, A, B, C, D) -> Fut + Send + Sync,
    Fut: Future<Output = Res> + Send,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
    D: IntoArgument<State>,
{
    fn run(&self, ctx: SlashContext<State>) -> DynFuture<'_, CommandResult> {
        Box::pin(async move {
            let options = ctx.event_data.options.clone();

            let a = parse_arg(ctx.clone(), &options, 0).await?;
            let b = parse_arg(ctx.clone(), &options, 1).await?;
            let c = parse_arg(ctx.clone(), &options, 2).await?;
            let d = parse_arg(ctx.clone(), &options, 3).await?;

            self(ctx, a, b, c, d).await;

            Ok(())
        })
    }

    fn argument_types(&self) -> Vec<ArgumentType> {
        vec![A::r#type(), B::r#type(), C::r#type(), D::r#type()]
    }
}

impl<State, Func, Fut, Res, A, B, C, D, E> SlashCommandHandler<State, (A, B, C, D, E)> for Func
where
    State: StateBound,
    Func: Fn(SlashContext<State>, A, B, C, D, E) -> Fut + Send + Sync,
    Fut: Future<Output = Res> + Send,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
    D: IntoArgument<State>,
    E: IntoArgument<State>,
{
    fn run(&self, ctx: SlashContext<State>) -> DynFuture<'_, CommandResult> {
        Box::pin(async move {
            let options = ctx.event_data.options.clone();

            let a = parse_arg(ctx.clone(), &options, 0).await?;
            let b = parse_arg(ctx.clone(), &options, 1).await?;
            let c = parse_arg(ctx.clone(), &options, 2).await?;
            let d = parse_arg(ctx.clone(), &options, 3).await?;
            let e = parse_arg(ctx.clone(), &options, 4).await?;

            self(ctx, a, b, c, d, e).await;

            Ok(())
        })
    }

    fn argument_types(&self) -> Vec<ArgumentType> {
        vec![
            A::r#type(),
            B::r#type(),
            C::r#type(),
            D::r#type(),
            E::r#type(),
        ]
    }
}

impl<State, Func, Fut, Res, A, B, C, D, E, F> SlashCommandHandler<State, (A, B, C, D, E, F)>
    for Func
where
    State: StateBound,
    Func: Fn(SlashContext<State>, A, B, C, D, E, F) -> Fut + Send + Sync,
    Fut: Future<Output = Res> + Send,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
    D: IntoArgument<State>,
    E: IntoArgument<State>,
    F: IntoArgument<State>,
{
    fn run(&self, ctx: SlashContext<State>) -> DynFuture<'_, CommandResult> {
        Box::pin(async move {
            let options = ctx.event_data.options.clone();

            let a = parse_arg(ctx.clone(), &options, 0).await?;
            let b = parse_arg(ctx.clone(), &options, 1).await?;
            let c = parse_arg(ctx.clone(), &options, 2).await?;
            let d = parse_arg(ctx.clone(), &options, 3).await?;
            let e = parse_arg(ctx.clone(), &options, 4).await?;
            let f = parse_arg(ctx.clone(), &options, 5).await?;

            self(ctx, a, b, c, d, e, f).await;

            Ok(())
        })
    }

    fn argument_types(&self) -> Vec<ArgumentType> {
        vec![
            A::r#type(),
            B::r#type(),
            C::r#type(),
            D::r#type(),
            E::r#type(),
            F::r#type(),
        ]
    }
}

/// A wrapper for slash command handlers that implements [`SlashCommandHandlerWithoutArgs`].
pub struct SlashCommandHandlerWrapper<F, Args>
where
    Args: Send + Sync,
{
    handler: F,
    __args: PhantomData<Args>,
}

impl<F, Args> SlashCommandHandlerWrapper<F, Args>
where
    Args: Send + Sync,
{
    fn new(handler: F) -> Self {
        SlashCommandHandlerWrapper {
            handler,
            __args: PhantomData,
        }
    }
}

/// A trait for all wrapped slash command handlers to be called without an args generic.
pub trait SlashCommandHandlerWithoutArgs<State>: Send + Sync
where
    State: StateBound,
{
    /// Proxies to [`SlashCommandHandler::run`].
    /// 
    /// Arguments:
    /// * `ctx` - The slash command context.
    /// 
    /// Returns:
    /// [`CommandResult`] - The result of running, or attempting to run, the command handler.
    fn run(&self, ctx: SlashContext<State>) -> DynFuture<'_, CommandResult>;

    /// A vector of argument types taken by this handler.
    /// 
    /// Return:
    /// [`Vec<ArgumentType>`] - The argument types taken by this handler.
    fn argument_types(&self) -> Vec<ArgumentType>;
}

impl<State, F, Args> SlashCommandHandlerWithoutArgs<State> for SlashCommandHandlerWrapper<F, Args>
where
    State: StateBound,
    F: SlashCommandHandler<State, Args>,
    Args: Send + Sync,
{
    fn run(&self, ctx: SlashContext<State>) -> DynFuture<'_, CommandResult> {
        SlashCommandHandler::run(&self.handler, ctx)
    }

    fn argument_types(&self) -> Vec<ArgumentType> {
        SlashCommandHandler::argument_types(&self.handler)
    }
}

/// A group of slash commands.
#[derive(Clone)]
pub struct SlashCommandGroup<State>
where
    State: StateBound,
{
    /// The command group's name.
    pub name: String,

    /// The command group's subcommands and subgroups.
    pub children: Vec<CommandNode<State>>,
}

impl<State> SlashCommandGroup<State>
where
    State: StateBound,
{
    pub fn build(name: impl Into<String>) -> SlashCommandGroupBuilder<State> {
        SlashCommandGroupBuilder::new(name)
    }
}

/// A slash command group builder, which allows setting extra metadata.
#[derive(Clone)]
pub struct SlashCommandGroupBuilder<State>
where
    State: StateBound,
{
    name: String,
    children: Vec<CommandNode<State>>,
}

impl<State> SlashCommandGroupBuilder<State>
where
    State: StateBound,
{
    pub(crate) fn new(name: impl Into<String>) -> Self {
        SlashCommandGroupBuilder {
            name: name.into(),
            children: vec![],
        }
    }

    /// Adds a command to the group.
    /// 
    /// Arguments:
    /// * `command` - The command to add to the command group.
    /// 
    /// Returns:
    /// [`SlashCommandGroupBuilder`] - The current builder, with the command set.
    pub fn command(mut self, command: impl Into<SlashCommand<State>>) -> Self {
        self.children
            .push(CommandNode::SlashCommand(command.into()));
        self
    }

    /// Nests a group into this group.
    /// 
    /// Arguments:
    /// * `group` - The group to nest.
    /// 
    /// Returns:
    /// [`SlashCommandGroupBuilder`] - The current builder with the nested group.
    pub fn nest(mut self, group: impl Into<SlashCommandGroup<State>>) -> Self {
        self.children
            .push(CommandNode::SlashCommandGroup(group.into()));
        self
    }

    pub(crate) fn build(self) -> SlashCommandGroup<State> {
        SlashCommandGroup {
            name: self.name,
            children: self.children,
        }
    }
}

impl<State> CommandGroupIntoCommandNode<State> for SlashCommandGroup<State>
where
    State: StateBound,
{
    fn into_command_node(self) -> CommandNode<State> {
        CommandNode::SlashCommandGroup(self)
    }
}

impl<State> CommandGroupIntoCommandNode<State> for SlashCommandGroupBuilder<State>
where
    State: StateBound,
{
    fn into_command_node(self) -> CommandNode<State> {
        CommandNode::SlashCommandGroup(self.build())
    }
}

/// An error occurred while validating the configured slash commands.
#[derive(Debug, Error)]
pub enum InvalidCommandError {
    #[error(
        "The command /{0}'s handler has more arguments than you defined when building the command through the `Command` interface. Add the remaining arguments."
    )]
    TooFewArguments(String),

    #[error(
        "The command /{0}'s handler has less arguments than you defined when building the command through the `Command` interface. Remove the extra arguments."
    )]
    TooManyArguments(String),

    #[error(
        "The command /{0} has invalid arguments passed as metadata. The handler function defines an argument of type {1:?}, but the metadata you set defines an argument of type {2:?}. This will always fail to parse. Correct the command's metadata, or change your handler's signature."
    )]
    MismatchingArgumentTypes(String, ArgumentType, ArgumentType),

    #[error("You have a slash command with an empty name!")]
    CommandNameTooShort,

    #[error("Your command /{0}'s name is too long. It cannot exceed 32 characters.")]
    CommandNameTooLong(String),

    #[error(
        "Your command /{0} has an argument whose name is empty. Set it to something less than 32 characters long."
    )]
    ArgumentNameTooShort(String),

    #[error(
        "Your command /{0} has an argument called {1} whose name is too long. Set it to something less than 32 characters long."
    )]
    ArgumentNameTooLong(String, String),
}

/// Validates a bot's slash commands and its arguments.
/// 
/// Arguments:
/// * `commands` - The slash commands the group has configured.
/// 
/// Returns:
/// * `Ok(())` - If all commands are valid.
/// * `Err(Vec<InvalidCommandError>)` - A list of errors found while validating the commands.
pub fn validate_commands<State>(
    commands: &[&SlashCommand<State>],
) -> Result<(), Vec<InvalidCommandError>>
where
    State: StateBound,
{
    let mut errors = vec![];

    for command in commands {
        let handler_argument_types = command.handler.argument_types();
        let command_argument_types = command
            .arguments
            .iter()
            .map(|a| a.r#type())
            .collect::<Vec<_>>();

        if handler_argument_types.len() > command_argument_types.len() {
            errors.push(InvalidCommandError::TooFewArguments(command.name.clone()));
        } else if handler_argument_types.len() < command_argument_types.len() {
            errors.push(InvalidCommandError::TooManyArguments(command.name.clone()));
        }

        for argument in &command.arguments {
            if argument.name().len() > 32 {
                errors.push(InvalidCommandError::ArgumentNameTooLong(
                    command.name.clone(),
                    argument.name().clone(),
                ));
            } else if argument.name().is_empty() {
                errors.push(InvalidCommandError::ArgumentNameTooShort(
                    command.name.clone(),
                ));
            }
        }

        for (type_1, type_2) in handler_argument_types.iter().zip(command_argument_types) {
            if *type_1 != type_2 {
                errors.push(InvalidCommandError::MismatchingArgumentTypes(
                    command.name.clone(),
                    *type_1,
                    type_2,
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
