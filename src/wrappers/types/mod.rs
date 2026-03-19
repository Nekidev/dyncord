//! A set of pretty wrappers around twilight types.
//!
//! The builders in this module are designed to provide a more ergonomic interface for creating and
//! sending messages, embeds, and other Discord objects. We do not currently have wrappers for all
//! Discord objects, but we plan to add more in the future.
//!
//! Many times, these builders can be used interchangeably with the core twilight types. Dyncord
//! functions take `impl Into<TwilightType>` for the relevant types, so it's up to you.
//!
//! You'll see many types here that don't have an obvious reason for being wrapped at first.
//! Usually, types wrapped here are either commonly-used types that get better ergonomics by being
//! wrapped, or types that can be cached and the wrappers implement the necessary traits for it.
//! 
//! Cacheable types are derived by [`serde::Serialize`] and [`serde::Deserialize`] when the feature
//! flag `cache-serde` is enabled, and by [`bitcode::Encode`] and [`bitcode::Decode`] when the
//! `cache-bitcode` feature flag is enabled.
//! 
//! If there's any specific object you'd like to see a builder for, please open an issue or submit
//! a pull request!

pub mod channels;
pub mod embeds;
pub mod roles;
pub mod users;
