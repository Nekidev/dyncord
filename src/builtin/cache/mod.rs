//! Built-in implementations of cache backends.
//!
//! Currently, there's only an in-memory cache backend. Check its [module documentation](inmemory)
//! to learn how to use it.

#[cfg(feature = "builtin-cache-inmemory")]
pub mod inmemory;
