//! A local in-memory cache backend.
//! 
//! To use this backend, add the `builtin-cache-inmemory` feature flag to dyncord and add the
//! backend to your [`Bot`](crate::Bot) as follows:
//! 
//! ```
//! let bot = Bot::new(()).with_cache(InMemoryCache::default());
//! ```
//! 
//! That's it! The cache will automatically process events as they're received and update its data.

use papaya::HashMap;

use crate::cache::{Cache, CacheError};
use crate::utils::{DynFuture, pinbox};
use crate::wrappers::types::users::User;

/// A local in-memory cache backend.
#[derive(Default)]
pub struct InMemoryCache {
    users_by_id: HashMap<u64, User>,
    users_by_name: HashMap<String, User>,
}

impl Cache for InMemoryCache {
    fn set_user(&self, user: User) -> DynFuture<'_, Result<(), CacheError>> {
        let pin_by_id = self.users_by_id.pin();
        let pin_by_name = self.users_by_name.pin();

        pin_by_id.insert(user.id, user.clone());
        pin_by_name.insert(user.name.clone(), user.clone());

        pinbox(Ok(()))
    }

    fn get_user_by_id(&self, user_id: u64) -> DynFuture<'_, Result<Option<User>, CacheError>> {
        let pin_by_id = self.users_by_id.pin();

        pinbox(Ok(pin_by_id.get(&user_id).cloned()))
    }

    fn get_user_by_name(
        &self,
        user_name: String,
    ) -> DynFuture<'_, Result<Option<User>, CacheError>> {
        let pin_by_name = self.users_by_name.pin();

        pinbox(Ok(pin_by_name.get(&user_name).cloned()))
    }
}
