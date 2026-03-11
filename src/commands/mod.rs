//! Message-based (prefixed) commands for [`Bot`](crate::Bot).
//!
//! Message commands in Dyncord are simple to create. They're in great part just a function that
//! takes [`CommandContext`] as its first argument and any* amount of arguments implementing
//! [`IntoArgument`] as the rest of the arguments. The return type can be anything convenient for
//! you as long as the function is asynchronous.
//!
//! A basic command looks like this:
//!
//! ```
//! async fn hello(ctx: CommandContext) {}
//! ```
//!
//! To add such command handler to your bot, just create a [`Command`] with the handler and add it
//! to your bot with [`Bot::command()`](crate::bot::Bot::command):
//!
//! ```
//! let bot = Bot::new(()).command(Command::new("hello", hello));
//! ```
//! 
//! You can also add aliases to your command, which are secondary names that can also be used to
//! invoke the command.
//! 
//! ```
//! let bot = Bot::new(()).command(Command::new("hello", hello).aliases(["hi", "hey"]));
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
//! async fn add(ctx: CommandContext, a: i32, b: i32, c: i32) {
//!     let sum = a + b + c;
//!
//!     ctx.send(format!("The answer is {sum}")).await.unwrap();
//! }
//! ```
//!
//! Currently, only a few types are supported as arguments natively:
//!
//! - `String` - A single word argument.
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
//! The [`CommandContext<State>`] is the required first argument of all command handlers. It takes
//! a generic `State` type which is the same as the one used in your bot, or `()` by default when
//! you don't need any state.
//! 
//! To send a message in the same channel the command was executed, use [`CommandContext::send()`]:
//! 
//! ```
//! async fn hello(ctx: CommandContext) {
//!     ctx.send("Hello, world!").await.unwrap();
//! }
//! ```

pub mod arguments;
pub mod context;
pub mod errors;
pub mod parsing;
pub mod prefixes;
pub(crate) mod event;

use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::DynFuture;
use crate::commands::arguments::IntoArgument;
use crate::commands::context::CommandContext;
use crate::commands::errors::CommandError;
use crate::state::StateBound;

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
pub struct Command<State>
where
    State: StateBound,
{
    /// The command's name, used to invoke the command.
    pub name: String,

    /// Secondary names that can also be used to invoke the command.
    pub aliases: Vec<String>,

    /// The command's handler, the function that executes when the command is run.
    pub(crate) handler: Arc<dyn CommandHandlerWithoutArgs<State>>,
}

impl<State> Command<State>
where
    State: StateBound,
{
    /// Creates a new command to route to.
    ///
    /// For example:
    /// ```rust
    /// async fn hello(ctx: Context, user: User) -> Result<(), CommandError> {
    ///     ctx.send(format!("Hey, {}!", user.name)).await?;
    ///
    ///     Ok(())
    /// }
    ///
    /// let bot = Bot::new().command(Command::new("hello", hello));
    /// bot.run("token").await.unwrap();
    /// ```
    ///
    /// Arguments:
    /// * `name` - The command's name, used to invoke the command.
    /// * `handler` - The command's handler, the function that executes when the command is run.
    ///
    /// Returns:
    /// [`Command`] - A new command to route to.
    pub fn new<F, Args>(name: impl Into<String>, handler: F) -> Self
    where
        F: CommandHandler<State, Args> + 'static,
        Args: Send + Sync + 'static,
    {
        let wrapper = CommandHandlerHolder::new(handler);

        Command {
            name: name.into(),
            aliases: Vec::new(),
            handler: Arc::new(wrapper),
        }
    }

    /// Adds aliases to the command.
    /// 
    /// Arguments:
    /// * `aliases` - The command's aliases, which are secondary names that can also be used to
    ///   invoke the command. It can be one or multiple aliases in various formats, such as a
    ///   single string, a vector of strings, an array of string slices, etc.
    /// 
    /// Returns:
    /// [`Command`] - The command with the added aliases.
    pub fn aliases(mut self, aliases: impl IntoAliases) -> Self {
        self.aliases.append(&mut aliases.into_aliases());
        self
    }

    /// Gets a list of all the command's identifiers, which are the command's name and its aliases.
    /// 
    /// Returns:
    /// `Vec<String>` - A list of all the command's identifiers.
    pub(crate) fn identifiers(&self) -> Vec<String> {
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
    pub(crate) async fn run(&self, ctx: CommandContext<State>, args: &str) {
        let _ = self.handler.run(ctx, args).await;
    }
}

pub type CommandResult = Result<(), CommandError>;

/// Trait for command handlers, the functions that execute when a command is run.
pub trait CommandHandler<State, Args>: Send + Sync
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
    fn run(&self, ctx: CommandContext<State>, args: &str) -> DynFuture<'_, CommandResult>;
}

impl<State, F, Fut, Res> CommandHandler<State, ()> for F
where
    F: Fn(CommandContext<State>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
{
    fn run(&self, ctx: CommandContext<State>, _args: &str) -> DynFuture<'_, CommandResult> {
        Box::pin(async move {
            (self)(ctx).await;

            Ok(())
        })
    }
}

impl<State, Func, Fut, A, Res> CommandHandler<State, (A,)> for Func
where
    Func: Fn(CommandContext<State>, A) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
{
    fn run(&self, ctx: CommandContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
        let args = args.to_string();

        Box::pin(async move {
            let (a, _remaining) = A::into_argument(ctx.clone(), args).await?;

            (self)(ctx, a).await;

            Ok(())
        })
    }
}

impl<State, Func, Fut, A, B, Res> CommandHandler<State, (A, B)> for Func
where
    Func: Fn(CommandContext<State>, A, B) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
{
    fn run(&self, ctx: CommandContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
        let args = args.to_string();

        Box::pin(async move {
            let (a, remaining) = A::into_argument(ctx.clone(), args).await?;
            let (b, _remaining) = B::into_argument(ctx.clone(), remaining).await?;

            (self)(ctx, a, b).await;

            Ok(())
        })
    }
}

impl<State, Func, Fut, A, B, C, Res> CommandHandler<State, (A, B, C)> for Func
where
    Func: Fn(CommandContext<State>, A, B, C) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
{
    fn run(&self, ctx: CommandContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
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

impl<State, Func, Fut, A, B, C, D, Res> CommandHandler<State, (A, B, C, D)> for Func
where
    Func: Fn(CommandContext<State>, A, B, C, D) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
    D: IntoArgument<State>,
{
    fn run(&self, ctx: CommandContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
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

impl<State, Func, Fut, A, B, C, D, E, Res> CommandHandler<State, (A, B, C, D, E)> for Func
where
    Func: Fn(CommandContext<State>, A, B, C, D, E) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    State: StateBound,
    A: IntoArgument<State>,
    B: IntoArgument<State>,
    C: IntoArgument<State>,
    D: IntoArgument<State>,
    E: IntoArgument<State>,
{
    fn run(&self, ctx: CommandContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
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
pub trait CommandHandlerWithoutArgs<State>: Send + Sync
where
    State: StateBound,
{
    fn run(&self, ctx: CommandContext<State>, args: &str) -> DynFuture<'_, CommandResult>;
}

impl<State, F, Args> CommandHandlerWithoutArgs<State> for CommandHandlerHolder<F, Args>
where
    State: StateBound,
    F: CommandHandler<State, Args>,
    Args: Send + Sync + 'static,
{
    fn run(&self, ctx: CommandContext<State>, args: &str) -> DynFuture<'_, CommandResult> {
        self.handler.run(ctx, args)
    }
}
