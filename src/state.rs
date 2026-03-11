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

/// Defines the boundaries for the state that can be used in a bot instance.
pub trait StateBound: Clone + Send + Sync + 'static {}

impl<T> StateBound for T where T: Clone + Send + Sync + 'static {}
