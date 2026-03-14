//! Message-based (prefixed) commands for [`Bot`](crate::Bot).
//!
//! Prefixed commands in Dyncord are simple to create. They're in great part just a function that
//! takes [`PrefixedContext`] as its first argument and any* amount of arguments implementing
//! [`IntoArgument`] as the rest of the arguments. The return type can be anything convenient for
//! you as long as the function is asynchronous.
//!
//! A basic command looks like this:
//!
//! ```
//! async fn hello(ctx: PrefixedContext) {}
//! ```
//!
//! To add such command handler to your bot, just create a [`Command`](crate::commands::Command)
//! with the handler and add it to your bot with [`Bot::command()`](crate::bot::Bot::command):
//!
//! ```
//! let bot = Bot::new(()).command(Command::prefixed("hello", hello));
//! ```
//!
//! You can also add aliases to your command, which are secondary names that can also be used to
//! invoke the command.
//!
//! ```
//! let bot = Bot::new(()).command(Command::prefixed("hello", hello).aliases(["hi", "hey"]));
//! ```
//!
//! # Intents
//!
//! To receive and respond to message comnands, bots need at least 2 intents: `MESSAGE_CONTENT` and
//! `GUILD_MESSAGES` or `DIRECT_MESSAGES` (or both) depending on where you want to receive
//! commands. You can add them to your bot with [`Bot::intents()`](crate::bot::Bot::intents):
//!
//! ```
//! Bot::new(()).intents(Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT);
//! ```
//!
//! # Prefixes
//!
//! By default, Dyncord doesn't use any prefix for message commands, so you need to set at least
//! one for it to start routing message commands. You can set a multiple prefixes and even compute
//! them dynamically based on the message and the bot's state. To set one prefix, just call
//! [`Bot::with_prefix()`](crate::bot::Bot::with_prefix) with the prefix you want to use:
//!
//! ```
//! Bot::new(()).with_prefix(".");
//! ```
//!
//! To set multiple prefixes, just call [`Bot::with_prefix()`](crate::bot::Bot::with_prefix) with a
//! more prefixes:
//!
//! ```
//! Bot::new(()).with_prefix([".", "!"]);
//! ```
//!
//! That'll make the bot listen for both `.` and `!` as prefixes. So, for example, if you have a
//! command named "hello", you can invoke it with either `.hello` or `!hello`.
//!
//! To compute prefixes dynamically, you can pass a function as an argument to
//! [`Bot::with_prefix()`](crate::bot::Bot::with_prefix). The function takes a
//! [`PrefixesContext`](prefixes::PrefixesContext) as an argument and returns a `Vec<String>` with
//! the prefixes to use for the message. For example:
//!
//! ```
//! async fn get_prefixes(ctx: PrefixesContext) -> Vec<String> {
//!     // Dummy code to get the prefixes from a hypothetical database.
//!     ctx.state.db.get_prefixes(ctx.event.guild_id.get()).await
//! }
//!
//! let bot = Bot::new(()).with_prefix(get_prefixes);
//! ```
//!
//! For more complex implementations of dynamic prefixes, you can also implement
//! [`Prefixes`](prefixes::Prefixes) for a type and pass an instance of it to
//! [`Bot::with_prefix()`](crate::bot::Bot::with_prefix) instead.
//!
//! # Arguments
//!
//! Parsing command arguments is easy with Dyncord. You just need to add them as arguments to your
//! command handler function and make sure their types implement [`IntoArgument`]. Dyncord will
//! take care of parsing the arguments from the raw string and passing them to your handler
//! properly.
//!
//! Arguments are usually delimited by a whitespace. For example, a command called "add" could take
//! 3 arguments like `!add 1 2 3`, and the handler would look like this:
//!
//! ```
//! async fn add(ctx: PrefixedContext, a: i32, b: i32, c: i32) {
//!     let sum = a + b + c;
//!
//!     ctx.send(format!("The answer is {sum}")).await.unwrap();
//! }
//! ```
//!
//! Currently, only a few types are supported as arguments natively:
//!
//! - `String` - A single-word argument, or a quoted string. E.g. `!command hello` -> `hello`,
//!   `!command hello world` -> `hello`, `!command "hello world"` -> `hello world`,
//!   `!command 'Someone\'s cat'` -> `Someone's cat`, `!command 'hello world` -> `'hello`,
//!   `!command \'hello\'` -> `'hello'`.
//! - [`GreedyString`](arguments::GreedyString) - A string that consumes all remaining raw
//!   arguments.
//! - `char` - A single character argument. The argument must be a single character long, otherwise
//!   it'll fail to parse.
//! - `i8`, `i16`, `i32`, `i64`, `i128`, `isize` - Signed integer arguments.
//! - `u8`, `u16`, `u32`, `u64`, `u128`, `usize` - Unsigned integer arguments.
//! - `f32`, `f64` - Floating point arguments.
//! - `bool` - A boolean argument. "true" | "y" | "yes" | "1" | "on" are considered `true`, while
//!   "false" | "n" | "no" | "0" | "off" are considered `false`.
//! - `Option<T>` - An optional argument. If the argument fails to parse, it'll be considered
//!   `None` instead of stopping the command's execution.
//!
//! Writing a custom argument type is also easy. You just need to implement [`IntoArgument`] for
//! your type and add the parsing logic in the `into_argument` function.
//!
//! For example, let's create a custom `Name` argument type that takes two words as the first and
//! last name of a person:
//!
//! ```
//! struct Name(String, String);
//!
//! impl IntoArgument<()> for Name {
//!     fn into_argument(
//!         _ctx: CommandContext,
//!         args: String,
//!     ) -> dyncord::DynFuture<'static, Result<(Self, String), ParsingError>> {
//!         Box::pin(async move {
//!             // Collect into a vector of the first up to 3 parts of the arguments, since we need
//!             // 2 words for the name and the rest of the raw arguments.
//!             let mut parts = args.splitn(3, ' ').collect::<Vec<&str>>();
//!
//!             // It must have a first and last name, otherwise it's invalid.
//!             if parts.len() < 2 {
//!                 return Err(ParsingError::InvalidArgument);
//!             }
//!
//!             // Only 2 words, meaning nothing remains. We add an empty string as the remaining
//!             // raw arguments.
//!             if parts.len() == 2 {
//!                 parts.push("");
//!             }
//!
//!             Ok((
//!                 Name(parts[0].trim().to_string(), parts[1].trim().to_string()),
//!                 parts[2].to_string(),
//!             ))
//!         })
//!     }
//! }
//! ```
//!
//! # Command Context
//!
//! The command context is a struct that contains information about the context in which the
//! command is being executed. This includes things like the event that triggered the command, the
//! prefix used, the author, channel, guild, your bot's state, and more. It also provides some
//! utility functions to interact with the Discord API, such as sending messages.
//!
//! The [`PrefixedContext<State>`] is the required first argument of all command handlers. It takes
//! a generic `State` type which is the same as the one used in your bot, or `()` by default when
//! you don't need any state.
//!
//! To send a message in the same channel the command was executed, use [`PrefixedContext::send()`]:
//!
//! ```
//! async fn hello(ctx: PrefixedContext) {
//!     ctx.send("Hello, world!").await.unwrap();
//! }
//! ```
//!
//! # Command Groups
//!
//! When your bot starts to grow and you have many commands, it can be useful to group them
//! together by utility. For example, you may want to group admin commands together and
//! entertainment commands together. Dyncord supports this with command groups, which are just a
//! collection of commands, and optionally more subgroups.
//!
//! You can create a [`CommandGroup`](crate::commands::CommandGroup) with similar fields to
//! [`Command`](crate::commands::Command), and add commands to it like you do to your
//! [`Bot`](crate::Bot).
//!
//! ```
//! let bot = Bot::new(())
//!     .command(Command::prefixed("help", help_command))
//!     .nest(
//!         CommandGroup::prefixed("admin")
//!             .command(Command::prefixed("ban", ban_command))
//!             .command(Command::prefixed("kick", kick_command))
//!             .command(Command::prefixed("mute", mute_command))
//!     )
//!     .nest(
//!         CommandGroup::prefixed("fun")
//!             .command(Command::prefixed("joke", joke_command))
//!             .command(Command::prefixed("meme", meme_command))
//!     );
//! ```
//!
//! Command groups don't change the functionality nor execution of such commands. They're only
//! grouped internally, and you can use such grouping to display them more organizedly in a help
//! command.

pub mod arguments;
pub mod context;
pub mod errors;
pub mod parsing;
pub mod prefixes;
pub(crate) mod routing;

use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::commands::prefixed::arguments::IntoArgument;
use crate::commands::prefixed::context::PrefixedContext;
use crate::commands::prefixed::errors::CommandError;
use crate::commands::{CommandGroupIntoCommandNode, CommandNode};
use crate::state::StateBound;
use crate::utils::DynFuture;

/// A trait for types that can be converted into a list of command aliases.
pub trait IntoAliases {
    fn into_aliases(self) -> Vec<String>;
}

impl IntoAliases for String {
    fn into_aliases(self) -> Vec<String> {
        vec![self]
    }
}

impl IntoAliases for &str {
    fn into_aliases(self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl IntoAliases for Vec<String> {
    fn into_aliases(self) -> Vec<String> {
        self
    }
}

impl IntoAliases for Vec<&str> {
    fn into_aliases(self) -> Vec<String> {
        self.into_iter().map(|s| s.to_string()).collect()
    }
}

impl IntoAliases for &[&str] {
    fn into_aliases(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}

impl<const N: usize> IntoAliases for [&str; N] {
    fn into_aliases(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}

/// A command that can be routed to when a message is received.
#[derive(Clone)]
pub struct PrefixedCommand<State>
where
    State: StateBound,
{
    /// The command's name, used to invoke the command.
    pub name: String,

    /// Secondary names that can also be used to invoke the command.
    pub aliases: Vec<String>,

    /// The command's summary. Ideally a short one-line description of the command.
    pub summary: Option<String>,

    /// The command's description. Ideally a more detailed description of the command, its
    /// arguments, and how to use it.
    pub description: Option<String>,

    /// The command's handler, the function that executes when the command is run.
    pub(crate) handler: Arc<dyn PrefixedCommandHandlerWithoutArgs<State>>,
}

impl<State> PrefixedCommand<State>
where
    State: StateBound,
{
    /// Creates a new command builder.
    ///
    /// For example:
    /// ```rust
    /// async fn hello(ctx: Context, user: User) -> Result<(), CommandError> {
    ///     ctx.send(format!("Hey, {}!", user.name)).await?;
    ///
    ///     Ok(())
    /// }
    ///
    /// let bot = Bot::new().command(Command::build("hello", hello));
    /// bot.run("token").await.unwrap();
    /// ```
    ///
    /// Arguments:
    /// * `name` - The command's name, used to invoke the command.
    /// * `handler` - The command's handler, the function that executes when the command is run.
    ///
    /// Returns:
    /// [`PrefixedCommandBuilder`] - A new command builder.
    pub fn build<F, Args>(name: impl Into<String>, handler: F) -> PrefixedCommandBuilder<State>
    where
        F: PrefixedCommandHandler<State, Args> + 'static,
        Args: Send + Sync + 'static,
    {
        PrefixedCommandBuilder::new(name, handler)
    }

    /// Gets a list of all the command's identifiers, which are the command's name and its aliases.
    ///
    /// Returns:
    /// `Vec<String>` - A list of all the command's identifiers.
    pub fn identifiers(&self) -> Vec<String> {
        let mut identifiers = vec![self.name.clone()];
        identifiers.extend(self.aliases.clone());
        identifiers
    }

    /// Runs the command handler.
    ///
    /// Arguments:
    /// * `ctx` - The context of the command, which contains information about the message,
    ///   channel, guild, etc.
    /// * `args` - The raw arguments passed to the command, which can be parsed into the command's
    ///   arguments.
    pub(crate) async fn run(&self, ctx: PrefixedContext<State>, args: &str) {
        let _ = self.handler.run(ctx, args).await;
    }
}

/// A builder for [`PrefixedCommand`] that allows setting optional fields like aliases, summary,
/// and description.
pub struct PrefixedCommandBuilder<State>
where
    State: StateBound,
{
    name: String,
    aliases: Vec<String>,
    summary: Option<String>,
    description: Option<String>,
    handler: Arc<dyn PrefixedCommandHandlerWithoutArgs<State>>,
}

impl<State> PrefixedCommandBuilder<State>
where
    State: StateBound,
{
    /// Creates a new command builder with the given name and handler.
    ///
    /// Arguments:
    /// * `name` - The command's name, used to invoke the command.
    /// * `handler` - The command's handler, the function that executes when the command is run.
    ///
    /// Returns:
    /// [`CommandBuilder`] - A new command builder with the given name and handler.    
    pub(crate) fn new<F, Args>(name: impl Into<String>, handler: F) -> Self
    where
        F: PrefixedCommandHandler<State, Args> + 'static,
        Args: Send + Sync + 'static,
    {
        let wrapper = CommandHandlerHolder::new(handler);

        PrefixedCommandBuilder {
            name: name.into(),
            aliases: Vec::new(),
            summary: None,
            description: None,
            handler: Arc::new(wrapper),
        }
    }

    /// Adds one or more aliases to the command.
    ///
    /// Arguments:
    /// * `aliases` - One or more aliases to add to the command. Can be a single string, a vector
    ///   of strings, or an array of string slices.
    ///
    /// Returns:
    /// [`PrefixedCommandBuilder`] - The command builder with the added aliases.
    pub fn aliases(mut self, aliases: impl IntoAliases) -> Self {
        self.aliases = aliases.into_aliases();
        self
    }

    /// Sets the command's summary, which is ideally a short one-line description of the command.
    ///
    /// Arguments:
    /// * `summary` - The command's summary.
    ///
    /// Returns:
    /// [`PrefixedCommandBuilder`] - The command builder with the set summary.
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Sets the command's description, which is ideally a more detailed description of the
    /// command.
    ///
    /// Arguments:
    /// * `description` - The command's description.
    ///
    /// Returns:
    /// [`PrefixedCommandBuilder`] - The command builder with the set description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Builds the command, consuming the builder and returning a [`Command`] with the set fields.
    ///
    /// Returns:
    /// [`PrefixedCommand`] - A command with the set fields from the builder.
    pub(crate) fn build(self) -> PrefixedCommand<State> {
        PrefixedCommand {
            name: self.name,
            aliases: self.aliases,
            summary: self.summary,
            description: self.description,
            handler: self.handler,
        }
    }
}

pub type CommandResult = Result<(), CommandError>;

/// Trait for command handlers, the functions that execute when a command is run.
pub trait PrefixedCommandHandler<State, Args>: Send + Sync
where
    State: StateBound,
{
    /// Runs the command handler.
    ///
    /// Arguments:
    /// * `ctx` - The context of the command, which contains information about the message,
    ///   channel, guild, etc.
    /// * `args` - The raw arguments passed to the command, which can be parsed into the command's
    ///   arguments.
    ///
    /// Returns:
    /// [`CommandResult`] - A future that resolves when the command handler has finished executing.
    /// This is equivalent to an asynchronous function, so you can just run `.run().await`.
    fn run(&self, ctx: PrefixedContext<State>, args: &str) -> DynFuture<'_, CommandResult>;
}

impl<State, F, Fut, Res> PrefixedCommandHandler<State, ()> for F
where
    F: Fn(PrefixedContext<State>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
{
    fn run(&self, ctx: PrefixedContext<State>, _args: &str) -> DynFuture<'_, CommandResult> {
        Box::pin(async move {
            (self)(ctx).await;

            Ok(())
        })
    }
}

impl<State, Func, Fut, A, Res> PrefixedCommandHandler<State, (A,)> for Func
where
    Func: Fn(PrefixedContext<State>, A) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
{
    fn run(&self, ctx: PrefixedContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
        let args = args.to_string();

        Box::pin(async move {
            let (a, _remaining) = A::into_argument(ctx.clone(), args).await?;

            (self)(ctx, a).await;

            Ok(())
        })
    }
}

impl<State, Func, Fut, A, B, Res> PrefixedCommandHandler<State, (A, B)> for Func
where
    Func: Fn(PrefixedContext<State>, A, B) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
{
    fn run(&self, ctx: PrefixedContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
        let args = args.to_string();

        Box::pin(async move {
            let (a, remaining) = A::into_argument(ctx.clone(), args).await?;
            let (b, _remaining) = B::into_argument(ctx.clone(), remaining).await?;

            (self)(ctx, a, b).await;

            Ok(())
        })
    }
}

impl<State, Func, Fut, A, B, C, Res> PrefixedCommandHandler<State, (A, B, C)> for Func
where
    Func: Fn(PrefixedContext<State>, A, B, C) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
{
    fn run(&self, ctx: PrefixedContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
        let args = args.to_string();

        Box::pin(async move {
            let (a, remaining) = A::into_argument(ctx.clone(), args).await?;
            let (b, remaining) = B::into_argument(ctx.clone(), remaining).await?;
            let (c, _remaining) = C::into_argument(ctx.clone(), remaining).await?;

            (self)(ctx, a, b, c).await;

            Ok(())
        })
    }
}

impl<State, Func, Fut, A, B, C, D, Res> PrefixedCommandHandler<State, (A, B, C, D)> for Func
where
    Func: Fn(PrefixedContext<State>, A, B, C, D) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
    D: IntoArgument<State>,
{
    fn run(&self, ctx: PrefixedContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
        let args = args.to_string();

        Box::pin(async move {
            let (a, remaining) = A::into_argument(ctx.clone(), args).await?;
            let (b, remaining) = B::into_argument(ctx.clone(), remaining).await?;
            let (c, remaining) = C::into_argument(ctx.clone(), remaining).await?;
            let (d, _remaining) = D::into_argument(ctx.clone(), remaining).await?;

            (self)(ctx, a, b, c, d).await;

            Ok(())
        })
    }
}

impl<State, Func, Fut, A, B, C, D, E, Res> PrefixedCommandHandler<State, (A, B, C, D, E)> for Func
where
    Func: Fn(PrefixedContext<State>, A, B, C, D, E) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
    D: IntoArgument<State>,
    E: IntoArgument<State>,
{
    fn run(&self, ctx: PrefixedContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
        let args = args.to_string();

        Box::pin(async move {
            let (a, remaining) = A::into_argument(ctx.clone(), args).await?;
            let (b, remaining) = B::into_argument(ctx.clone(), remaining).await?;
            let (c, remaining) = C::into_argument(ctx.clone(), remaining).await?;
            let (d, remaining) = D::into_argument(ctx.clone(), remaining).await?;
            let (e, _remaining) = E::into_argument(ctx.clone(), remaining).await?;

            (self)(ctx, a, b, c, d, e).await;

            Ok(())
        })
    }
}

impl<State, Func, Fut, A, B, C, D, E, F, Res> PrefixedCommandHandler<State, (A, B, C, D, E, F)>
    for Func
where
    Func: Fn(PrefixedContext<State>, A, B, C, D, E, F) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
    D: IntoArgument<State>,
    E: IntoArgument<State>,
    F: IntoArgument<State>,
{
    fn run(&self, ctx: PrefixedContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
        let args = args.to_string();

        Box::pin(async move {
            let (a, remaining) = A::into_argument(ctx.clone(), args).await?;
            let (b, remaining) = B::into_argument(ctx.clone(), remaining).await?;
            let (c, remaining) = C::into_argument(ctx.clone(), remaining).await?;
            let (d, remaining) = D::into_argument(ctx.clone(), remaining).await?;
            let (e, remaining) = E::into_argument(ctx.clone(), remaining).await?;
            let (f, _remaining) = F::into_argument(ctx.clone(), remaining).await?;

            (self)(ctx, a, b, c, d, e, f).await;

            Ok(())
        })
    }
}

/// A wrapper for command handler functions to be able to implement [`CommandHandlerWithoutArgs`]
/// for all command handlers independetnly of their `Args` type.
struct CommandHandlerHolder<F, Args> {
    handler: F,
    _args: PhantomData<Args>,
}

impl<F, Args> CommandHandlerHolder<F, Args> {
    fn new(handler: F) -> Self {
        CommandHandlerHolder {
            handler,
            _args: PhantomData,
        }
    }
}

/// A trait for command handlers that can be run without a generic `Args` type.
///
/// This is used to be able to store command handlers in a vector without having to worry about
/// their `Args` type, given such generic is just a dummy type used to avoid implementation
/// collisions.
pub trait PrefixedCommandHandlerWithoutArgs<State>: Send + Sync
where
    State: StateBound,
{
    fn run(&self, ctx: PrefixedContext<State>, args: &str) -> DynFuture<'_, CommandResult>;
}

impl<State, F, Args> PrefixedCommandHandlerWithoutArgs<State> for CommandHandlerHolder<F, Args>
where
    State: StateBound,
    F: PrefixedCommandHandler<State, Args>,
    Args: Send + Sync + 'static,
{
    fn run(&self, ctx: PrefixedContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
        self.handler.run(ctx, args)
    }
}

/// A group of commands, which can be used to organize commands into categories.
#[derive(Clone)]
pub struct PrefixedCommandGroup<State>
where
    State: StateBound,
{
    /// The name of the group.
    pub name: String,

    /// The group's summary.
    pub summary: Option<String>,

    /// The group's description.
    pub description: Option<String>,

    /// Sub-commands and sub-groups of this group.
    pub children: Vec<CommandNode<State>>,
}

impl<State> PrefixedCommandGroup<State>
where
    State: StateBound,
{
    /// Creates a new command group builder with the given group name.
    ///
    /// Arguments:
    /// * `name` - The name of the group.
    ///
    /// Returns:
    /// [`PrefixedCommandGroupBuilder`] - A new command group builder with the given name.
    pub fn build(name: impl Into<String>) -> PrefixedCommandGroupBuilder<State> {
        PrefixedCommandGroupBuilder::new(name)
    }
}

/// A command group builder that allows setting optional fields.
pub struct PrefixedCommandGroupBuilder<State>
where
    State: StateBound,
{
    name: String,
    summary: Option<String>,
    description: Option<String>,
    children: Vec<CommandNode<State>>,
}

impl<State> PrefixedCommandGroupBuilder<State>
where
    State: StateBound,
{
    /// Creates a new command group builder with the given name.
    ///
    /// Arguments:
    /// * `name` - The name of the group.
    ///
    /// Returns:
    /// [`PrefixedCommandGroupBuilder`] - A new command group builder with the given name.
    pub(crate) fn new(name: impl Into<String>) -> Self {
        PrefixedCommandGroupBuilder {
            name: name.into(),
            summary: None,
            description: None,
            children: Vec::new(),
        }
    }

    /// Sets the group's summary, which is ideally a short one-line description of the group.
    ///
    /// Arguments:
    /// * `summary` - The group's summary.
    ///
    /// Returns:
    /// [`PrefixedCommandGroupBuilder`] - The command group builder with the set summary.
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Sets the group's description, which is ideally a more detailed description of the group.
    ///
    /// Arguments:
    /// * `description` - The group's description.
    ///
    /// Returns:
    /// [`PrefixedCommandGroupBuilder`] - The command group builder with the set description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a command as a child of this group.
    ///
    /// Arguments:
    /// * `command` - The command to add as a child of this group.
    ///
    /// Returns:
    /// [`PrefixedCommandGroupBuilder`] - The command group builder with the added command.
    pub fn command(mut self, command: PrefixedCommandBuilder<State>) -> Self {
        self.children
            .push(CommandNode::PrefixedCommand(command.build()));
        self
    }

    /// Adds a subgroup as a child of this group.
    ///
    /// Arguments:
    /// * `group` - The subgroup to add as a child of this group.
    ///
    /// Returns:
    /// [`PrefixedCommandGroupBuilder`] - The command group builder with the added subgroup.
    pub fn nest(mut self, group: PrefixedCommandGroupBuilder<State>) -> Self {
        self.children
            .push(CommandNode::PrefixedCommandGroup(group.build()));
        self
    }

    /// Builds the command group, consuming the builder and returning a [`PrefixedCommandGroup`]
    /// with the set fields.
    ///
    /// Returns:
    /// [`PrefixedCommandGroup`] - A command group with the set fields from the builder.
    pub(crate) fn build(self) -> PrefixedCommandGroup<State> {
        PrefixedCommandGroup {
            name: self.name,
            summary: self.summary,
            description: self.description,
            children: self.children,
        }
    }
}

impl<State> CommandGroupIntoCommandNode<State> for PrefixedCommandGroup<State>
where
    State: StateBound,
{
    fn into_command_node(self) -> CommandNode<State> {
        CommandNode::PrefixedCommandGroup(self)
    }
}

impl<State> CommandGroupIntoCommandNode<State> for PrefixedCommandGroupBuilder<State>
where
    State: StateBound,
{
    fn into_command_node(self) -> CommandNode<State> {
        CommandNode::PrefixedCommandGroup(self.build())
    }
}
