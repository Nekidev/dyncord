//! A Redis-backed cache backend.
//!
//! To use this backend, enable the `builtin-cache-redis` feature flag and install the [`redis`]
//! crate with async support and tokio compatibility. You can do so by copy-pasting the following
//! in your terminal:
//!
//! ```sh
//! cargo add dyncord -F builtin-cache-redis
//! cargo add redis -F aio,tokio-comp
//! ```
//!
//! Once they're installed, connect to Redis using the [`redis`] crate. [`RedisCache`] supports
//! using either a [`MultiplexedConnection`], a [`ConnectionManager`], or a [`ClusterConnection`].
//!
//! # Connecting to Redis
//! 
//! [`RedisCache`] supports both single-node setups and cluster setups.
//! 
//! ## Connecting to a Single Node
//! 
//! To connect to a single node, use either [`ConnectionManager`] or [`MultiplexedConnection`].
//! [`ConnectionManager`] is recommended due to the automatic reconnection support if it
//! disconnects, unlike [`MultiplexedConnection`].
//! 
//! For this, you'll also need to add the `connection-manager` feature flag to the [`redis`] crate:
//! 
//! ```sh
//! cargo add redis -F connection-manager
//! ```
//! 
//! Then initialize a client like follows:
//!
//! ```
//! let client = Client::open("redis://localhost/").unwrap();
//! let connection = ConnectionManager::new(client).await.unwrap();
//! 
//! let cache = RedisCache::new(connection);
//! ```
//! 
//! ## Connecting to a Cluster
//! 
//! To connect to a single node, you'll need a [`ClusterConnection`]. For this, you'll also need
//! the `cluster-async` feature flag enabled in the [`redis`] crate:
//! 
//! ```sh
//! cargo add redis -F cluster-async
//! ```
//! 
//! Then intialize a [`ClusterClient`](redis::cluster::ClusterClient) and a [`ClusterConnection`]
//! like follows:
//! 
//! ```
//! let client = ClusterClient::new(vec!["redis://node1", "redis://node2"]).unwrap();
//! let connection = client.get_async_connection().await?;
//! 
//! let cache = RedisCache::new(connection);
//! ```
//! 
//! # Using the Cache
//! 
//! Once you have initialized your cache, you only need to pass it to your [`Bot`](crate::Bot).
//! 
//! ```
//! let cache = RedisCache::new(connection);
//! 
//! let bot = Bot::new(()).with_cache(cache);
//! ```
//! 
//! That's it! Dyncord will now automatically use Redis for caching.

use redis::AsyncCommands;
use redis::aio::{ConnectionLike, ConnectionManager, MultiplexedConnection};
use redis::cluster_async::ClusterConnection;

use crate::cache::{Cache, CacheError};
use crate::utils::DynFuture;
use crate::wrappers::types::users::User;

/// A Redis-backed cache backend.
pub struct RedisCache {
    connection: RedisConnection,
}

impl RedisCache {
    /// Initializes the cache backend from a Redis connection.
    ///
    /// Arguments:
    /// * `connection` - Either a [`MultiplexedConnection`], a [`ConnectionManager`], or a
    ///   [`ClusterConnection`].
    ///
    /// Returns:
    /// [`RedisCache`] - An initialized Redis cache instance.
    pub fn new(connection: impl Into<RedisConnection>) -> Self {
        Self {
            connection: connection.into(),
        }
    }
}

#[derive(Clone)]
pub enum RedisConnection {
    Cluster(ClusterConnection),
    Manager(ConnectionManager),
    Multiplexed(MultiplexedConnection),
}

impl From<ClusterConnection> for RedisConnection {
    fn from(value: ClusterConnection) -> Self {
        Self::Cluster(value)
    }
}

impl From<ConnectionManager> for RedisConnection {
    fn from(value: ConnectionManager) -> Self {
        Self::Manager(value)
    }
}

impl From<MultiplexedConnection> for RedisConnection {
    fn from(value: MultiplexedConnection) -> Self {
        Self::Multiplexed(value)
    }
}

impl ConnectionLike for RedisConnection {
    fn get_db(&self) -> i64 {
        match self {
            Self::Cluster(connection) => connection.get_db(),
            Self::Manager(connection) => connection.get_db(),
            Self::Multiplexed(connection) => connection.get_db(),
        }
    }

    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        match self {
            Self::Cluster(connection) => connection.req_packed_command(cmd),
            Self::Manager(connection) => connection.req_packed_command(cmd),
            Self::Multiplexed(connection) => connection.req_packed_command(cmd),
        }
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        match self {
            Self::Cluster(connection) => connection.req_packed_commands(cmd, offset, count),
            Self::Manager(connection) => connection.req_packed_commands(cmd, offset, count),
            Self::Multiplexed(connection) => connection.req_packed_commands(cmd, offset, count),
        }
    }
}

impl Cache for RedisCache {
    fn set_user(&self, user: User) -> DynFuture<'_, Result<(), CacheError>> {
        let mut conn = self.connection.clone();

        Box::pin(async move {
            let raw = bitcode::encode(&user);

            let _: () = conn.set(format!("user:id:{}", user.id), &raw).await?;
            let _: () = conn.set(format!("user:name:{}", user.name), &raw).await?;

            Ok(())
        })
    }

    fn get_user_by_id(&self, user_id: u64) -> DynFuture<'_, Result<Option<User>, CacheError>> {
        let mut conn = self.connection.clone();

        Box::pin(async move {
            let key = format!("user:id:{user_id}");

            let result: Option<Vec<u8>> = conn.get(key).await?;

            if let Some(result) = result {
                Ok(Some(bitcode::decode(&result)?))
            } else {
                Ok(None)
            }
        })
    }

    fn get_user_by_name(
        &self,
        user_name: String,
    ) -> DynFuture<'_, Result<Option<User>, CacheError>> {
        let mut conn = self.connection.clone();

        Box::pin(async move {
            let key = format!("user:name:{user_name}");

            let result: Option<Vec<u8>> = conn.get(key).await?;

            if let Some(result) = result {
                Ok(Some(bitcode::decode(&result)?))
            } else {
                Ok(None)
            }
        })
    }
}
