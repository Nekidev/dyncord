//! Custom state types that can be used across a bot instance.
//! 
//! This module only contains the [`StateBound`] trait, which defines the boundaries for state
//! types that can be used in a bot instance. Those are `Clone + Send + Sync + 'static`.
//! 
//! It's common to use this state to store things like database connections, caches, and any other
//! data that you want to be accessible everywhere in your bot. It's also common to wrap such state
//! in an [`Arc`](std::sync::Arc) to make cloning cheap (because your state is constantly being
//! cloned and sent across threads).
//! 
//! For example, you can define a custom state type like this:
//! 
//! ```
//! use std::sync::Arc;
//! 
//! struct AppState {
//!     db: DatabaseConnection,
//!     cache: Cache,
//! }
//! 
//! impl AppState {
//!     fn new(db: DatabaseConnection, cache: Cache) -> State {
//!         Arc::new(AppState { db, cache })
//!     }
//! }
//! 
//! type State = Arc<AppState>;
//! ```
//! 
//! And then use it in your bot instance like this:
//! 
//! ```
//! let state = AppState::new(db_connection, cache);
//! let bot = Bot::new(state);
//! ```
//! 
//! Note that once you customize your bot's state type, you need to pass it as a generic parameter
//! to all contexts and handlers. For example, in a command handler and an event handler:
//! 
//! ```
//! use std::sync::Arc;
//! 
//! struct AppState {
//!     db: DatabaseConnection,
//!     cache: Cache,
//! }
//! 
//! type State = Arc<AppState>;
//! 
//! async fn command(ctx: CommandContext<State>) {
//!     // ...
//! }
//! 
//! async fn on_message(ctx: EventContext<State, MessageCreate>) {
//!     // ...
//! }
//! ```
//! 
//! # Example
//! 
//! To demonstrate how to use custom states, let's build a bot that keeps track of how many
//! messages it has received.
//! 
//! To start with, let's define our custom state type. We'll use an [`Arc`](std::sync::Arc) holding
//! an [`AtomicUsize`](std::sync::atomic::AtomicUsize) to keep track of the message count in a
//! thread-safe way.
//! 
//! ```
//! #[derive(Default, Clone)]
//! struct CounterState {
//!     counter: Arc<AtomicUsize>,
//! }
//! ```
//! 
//! Great! Now, let's create a bot instance using this custom state. We'll initialize the counter
//! to zero, which is the default.
//! 
//! ```
//! let bot = Bot::new(CounterState::default());
//! ```
//! 
//! To be able to count the messages and to run commands, we need the message content and guild
//! message intents. So, let's make sure to enable those when creating the bot instance:
//! 
//! ```
//! let bot = Bot::new(CounterState::default())
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT);
//! ```
//! 
//! Let's now add an event handler to count the messages. We'll listen for the `MessageCreate`
//! event and increment our counter every time a message is created.
//! 
//! ```
//! async fn on_message(ctx: EventContext<CounterState, MessageCreate>) {
//!     ctx.state.counter.fetch_add(1, Ordering::SeqCst);
//! }
//! 
//! let bot = Bot::new(CounterState::default())
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .on_event(On::message_create(on_message));  // Don't forget to add the event handler to the bot instance!
//! ```
//! 
//! Finally, let's add a command to check the current message count. We'll create a simple command
//! called `!count` that responds with the current count of messages.
//! 
//! ```
//! async fn count_command(ctx: CommandContext<CounterState>) {
//!     let count = ctx.state.counter.load(Ordering::SeqCst);
//!     ctx.reply(format!("Message count: {}", count)).await.unwrap();
//! }
//!```
//! 
//! Let's add that command to the bot instance and set a prefix, and that should be it.
//! 
//! ```
//! let bot = Bot::new(CounterState::default())
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .with_prefix("!")
//!     .command(Command::new("count", count_command))
//!     .on_event(On::message_create(on_message));
//! 
//! bot.run("your_token_here").await.unwrap();
//! ```
//! 
//! That's it! Try sending some messages in a channel where the bot is present and then use the
//! `!count` command to see the state updating. Nice, isn't it?

/// Defines the boundaries for the state that can be used in a bot instance.
pub trait StateBound: Clone + Send + Sync + 'static {}

impl<T> StateBound for T where T: Clone + Send + Sync + 'static {}
