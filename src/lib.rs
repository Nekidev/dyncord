#![doc = include_str!("../README.md")]

mod aliases;
mod bot;
pub mod builtin;
pub mod commands;
pub mod events;
pub mod handle;
pub mod state;
pub mod utils;
pub mod wrappers;

pub use bot::Bot;
pub use twilight_gateway::Intents;
