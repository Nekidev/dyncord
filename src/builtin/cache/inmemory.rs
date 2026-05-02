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
use crate::wrappers::types::members::Member;
use crate::wrappers::types::servers::Server;
use crate::wrappers::types::users::User;

/// A local in-memory cache backend.
#[derive(Default)]
pub struct InMemoryCache {
    users_by_id: HashMap<u64, User>,
    users_by_name: HashMap<String, User>,
    servers_by_id: HashMap<u64, Server>,
    members_by_id: HashMap<(u64, u64), Member>,
}

impl Cache for InMemoryCache {
    fn set_user(&self, user: User) -> DynFuture<'_, Result<(), CacheError>> {
        let pin_by_id = self.users_by_id.pin();
        let pin_by_name = self.users_by_name.pin();

        if let Some(cached_user) = pin_by_id.get(&user.id)
            && cached_user.name != user.name
        {
            pin_by_name.remove(&cached_user.name);
        }

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

    fn set_server(&self, server: Server) -> DynFuture<'_, Result<(), CacheError>> {
        let pin_by_id = self.servers_by_id.pin();

        pin_by_id.insert(server.id, server);

        pinbox(Ok(()))
    }

    fn del_server(&self, server_id: u64) -> DynFuture<'_, Result<(), CacheError>> {
        let pin_by_id = self.servers_by_id.pin();

        pin_by_id.remove(&server_id);

        pinbox(Ok(()))
    }

    fn get_server_by_id(
        &self,
        server_id: u64,
    ) -> DynFuture<'_, Result<Option<Server>, CacheError>> {
        let pin_by_id = self.servers_by_id.pin();

        pinbox(Ok(pin_by_id.get(&server_id).cloned()))
    }

    fn set_member(&self, server_id: u64, member: Member) -> DynFuture<'_, Result<(), CacheError>> {
        let pin_by_id = self.members_by_id.pin();

        pin_by_id.insert((server_id, member.id), member);

        pinbox(Ok(()))
    }

    fn del_member(&self, server_id: u64, member_id: u64) -> DynFuture<'_, Result<(), CacheError>> {
        let pin_by_id = self.members_by_id.pin();

        pin_by_id.remove(&(server_id, member_id));

        pinbox(Ok(()))
    }

    fn get_member_by_id(
        &self,
        server_id: u64,
        member_id: u64,
    ) -> DynFuture<'_, Result<Option<Member>, CacheError>> {
        let pin_by_id = self.members_by_id.pin();

        pinbox(Ok(pin_by_id.get(&(server_id, member_id)).cloned()))
    }
}
