//! A set of pretty wrappers around twilight types.
//! 
//! The builders in this module are designed to provide a more ergonomic interface for creating and
//! sending messages, embeds, and other Discord objects. We do not currently have wrappers for all
//! Discord objects, but we plan to add more in the future.
//! 
//! All of these builders can be used interchangeably with the core twilight types. Dyncord
//! functions take `impl Into<TwilightType>` for the relevant types, so it's up to you.
//! 
//! If there's any specific object you'd like to see a builder for, please open an issue or submit
//! a pull request!

pub mod embeds;
