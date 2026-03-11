#![doc = include_str!("../README.md")]

use std::pin::Pin;

mod bot;
pub mod commands;
pub mod events;
pub mod handle;
pub mod state;

pub use bot::Bot;
pub use twilight_gateway::Intents;

/// A untility alias to boxed `Send + Sync` futures.
pub type DynFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
