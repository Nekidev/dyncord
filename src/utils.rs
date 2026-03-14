//! Utilities to work with dyncord types.

use std::pin::Pin;

/// Takes a value and returns it as a pinned and boxed future that awaits to it.
///
/// Arguments:
/// * `value` - The value to pin and box in a future.
///
/// Returns:
/// [`Pin<Box<dyn Future<Output = T> + Send + 'static>>`]
#[inline]
pub fn pinbox<T: Send + 'static>(value: T) -> Pin<Box<dyn Future<Output = T> + Send + 'static>> {
    Box::pin(async move { value })
}

/// A utility alias to boxed `Send + Sync` futures.
pub type DynFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
