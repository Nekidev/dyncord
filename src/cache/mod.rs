//! Caching for Discord API resources.
//!
//! Caching is an important part of any Discord application that wants to scale and be fast.
//! Without caching, you'll find yourself making potentially-slow queries to the Discord API way
//! more than you could be doing it and hitting rate limits sooner than necessary.
//!
//! Cache backends store Discord API data in a data store and let the bot query it without
//! interacting with the Discord API, therefore being faster and optimizing API calls. Dyncord
//! comes with opt-in built-in cache backends for you to get started quickly, but you can also
//! easily bring your own cache storages.
//!
//! # Built-In Cache Backends
//!
//! Built-in cache backends are in the [`builtin::cache`](crate::builtin::cache) module. Currently,
//! there's only one backend built-in. The in-memory backend.
//!
//! Built-in caches are opted out by default. For the built-in cache backend you want to use, add
//! the following feature flag:
//!
//! - In-Memory Backend: `builtin-cache-inmemory`
//!
//! Then, import them from their submodule within [`builtin::cache`](crate::builtin::cache). For
//! example,
//!
//! ```
//! use dyncord::builtin::cache::inmemory::InMemoryCache;
//! ```
//!
//! Then, add it to your bot like follows:
//!
//! ```
//! let bot = Bot::new(()).with_cache(InMemoryCache);
//! ```
//!
//! That's it! Dyncord will now automatically the cache backend everywhere it can be used.
//!
//! # Custom Cache backends
//!
//! You can also write your own custom cache backends, and Dyncord makes it easy to do so. All
//! cacheable types support both [`serde`] serialization and deserialization and [`bitcode`]
//! encoding and decoding. To enable support for each library, add the corresponding feature flag
//! to dyncord:
//!
//! - `cache-serde`
//! - `cache-bitcode`
//!
//! The feature flag you add will enable support for the library you will use.
//!
//! Creating a custom cache backend only requires to implement [`Cache`] on a custom type. In
//! essence,
//!
//! ```
//! struct HatsuneMikuCache;
//!
//! impl Cache for HatsuneMikuCache {
//!     ...
//! }
//! ```
//!
//! Each function the [`Cache`] trait has must be implemented. They all return a [`DynFuture`]
//! future, which in practice is just a pin-boxed future with some bounds. Instead of writing
//! `async fn`s, you'll write
//!
//! ```
//! fn cache_function(&self, ...) -> DynFuture<'_, Result<..., CacheError>> {
//!     Box::pin(async move {
//!         ...
//!
//!         Ok(())
//!     })
//! }
//! ```
//!
//! For an actual example of how an implemented cache backend looks like, you can look at the basic
//! [`InMemoryCache`](crate::builtin::cache::inmemory::InMemoryCache). It doesn't encode data, but
//! it implements all the methods you'll have to implement for your custom backend.
//!
//! ## Encoding Cacheable Data
//!
//! As described above, you can enable dyncord feature flags to get either [`serde`] or [`bitcode`]
//! encoding support on cacheable types.
//!
//! When `cache-serde` is enabled, you'll also need a serde serialization library to serialize and
//! deserialize types to the format you want. For example, using `serde_json` to store a [`User`],
//! you'd do something like
//!
//! ```
//! impl Cache for YourCacheBackend {
//!     fn set_user(&self, user: User) -> DynFuture<'_, Result<(), CacheError>> {
//!         Box::pin(async move {
//!             let encoded: String = serde_json::to_string(&user)?;
//!             
//!             // Now store the JSON in your custom backend...
//!
//!             Ok(())
//!         })
//!     }
//! }
//! ```
//!
//! Encoding using [`bitcode`] is practically the same, only that you don't have custom format
//! libraries and instead you use [`bitcode`] for encoding and decoding directly. For example,
//!
//! ```
//! impl Cache for YourCacheBackend {
//!     fn set_user(&self, user: User) -> DynFuture<'_, Result<(), CacheError>> {
//!         Box::pin(async move {
//!             let encoded: Vec<u8> = bitcode::encode(&user);
//!             
//!             // Now store the binary in your custom backend...
//!
//!             Ok(())
//!         })
//!     }
//! }
//! ```
//!
//! ## Using Neither Serde nor Bitcode
//!
//! If you don't want to use Serde nor Bitcode, you're going to have to wrap dyncord's types with
//! your own type wrappers and implement storage on those types, plus converting to and from
//! dyncord's wrapper types. It's a tedious task, so if you can do it with the dyncord-provided
//! types and either [`serde`] or [`bitcode`], it's way easier to do so.
//!
//! ## Storing Only Some Resources
//!
//! Your cache backend doesn't have to store every resource it's given to store. For example, if
//! your bot doesn't use message data at all, storing it in cache would only be a waste of space.
//!
//! Ignoring resource types is simple, just make the [`Cache`] associated functions related to the
//! type be no-ops. Setter functions always succeed but don't store anything, and getter functions
//! always succeed but find nothing.
//!
//! For example, to ignore [`User`] resources, make your [`Cache`] associated functions look like
//! this:
//!
//! ```
//! impl Cache for YourCacheBackend {
//!     fn set_user(&self, user: User) -> DynFuture<'_, Result<(), CacheError>> {
//!         pinbox(Ok(()))
//!     }
//!
//!     fn get_user_by_id(&self, user_id: u64) -> DynFuture<'_, Result<Option<User>, CacheError>> {
//!         pinbox(Ok(None))
//!     }
//!
//!     fn get_user_by_name(&self, user_name: String) -> DynFuture<'_, Result<Option<User>, CacheError>> {
//!         pinbox(Ok(None))
//!     }
//! }
//! ```

use std::error::Error;
use std::sync::Arc;

use twilight_gateway::Event;
use twilight_model::gateway::presence::UserOrId;

use crate::utils::DynFuture;
use crate::wrappers::types::users::User;

/// An error that occurred while on a cache operation.
pub type CacheError = Box<dyn Error + Send + Sync>;

/// Trait implemented by cache backends.
pub trait Cache: Send + Sync {
    /// Saves a user in cache.
    ///
    /// Arguments:
    /// * `user` - The user data to store in cache.
    ///
    /// Returns:
    /// * `Ok(())` - If no error occurred.
    /// * `Err(Error)` - If an error occurred.
    fn set_user(&self, user: User) -> DynFuture<'_, Result<(), CacheError>>;

    /// Gets a user from cache by ID.
    ///
    /// Arguments:
    /// * `user_id` - The ID of the user to get.
    ///
    /// Returns:
    /// * `Ok(None)` - No error occurred, the user was not in cache.
    /// * `Ok(Some(User))` - No error occurred, the user was found in cache.
    /// * `Err(Error)` - An error occurred while trying to get the user from cache.
    fn get_user_by_id(&self, user_id: u64) -> DynFuture<'_, Result<Option<User>, CacheError>>;

    /// Gets a user from cache by their username.
    ///
    /// Arguments:
    /// * `user_name` - The user's name.
    ///
    /// Returns:
    /// * `Ok(None)` - No error occurred, the user was not in cache.
    /// * `Ok(Some(User))` - No error occurred, the user was found in cache.
    /// * `Err(Error)` - An error occurred while trying to get the user from cache.
    fn get_user_by_name(
        &self,
        user_name: String,
    ) -> DynFuture<'_, Result<Option<User>, CacheError>>;
}

/// Processes an event received from the gateway, storing any found useful resources in cache.
///
/// Arguments:
/// * `event` - The event to process.
/// * `cache` - The cache backend to process it with.
///
/// Returns:
/// * `Ok(())` - No error occurred.
/// * `Err(Error)` - The cache backend returned an error.
pub(crate) async fn process_event(event: Event, cache: &dyn Cache) -> Result<(), CacheError> {
    match event {
        Event::AutoModerationActionExecution(_) => {}
        Event::AutoModerationRuleCreate(_) => {}
        Event::AutoModerationRuleDelete(_) => {}
        Event::AutoModerationRuleUpdate(_) => {}
        Event::BanAdd(event) => {
            cache.set_user(event.user.into()).await?;
        }
        Event::BanRemove(event) => {
            cache.set_user(event.user.into()).await?;
        }
        Event::ChannelCreate(_) => {}
        Event::ChannelDelete(_) => {}
        Event::ChannelPinsUpdate(_) => {}
        Event::ChannelUpdate(_) => {}
        Event::CommandPermissionsUpdate(_) => {}
        Event::EntitlementCreate(_) => {}
        Event::EntitlementDelete(_) => {}
        Event::EntitlementUpdate(_) => {}
        Event::GatewayClose(_) => {}
        Event::GatewayHeartbeat => {}
        Event::GatewayHeartbeatAck => {}
        Event::GatewayHello(_) => {}
        Event::GatewayInvalidateSession(_) => {}
        Event::GatewayReconnect => {}
        Event::GuildAuditLogEntryCreate(_) => {}
        Event::GuildCreate(_) => {}
        Event::GuildDelete(_) => {}
        Event::GuildEmojisUpdate(_) => {}
        Event::GuildIntegrationsUpdate(_) => {}
        Event::GuildScheduledEventCreate(event) => {
            if let Some(user) = event.creator.clone() {
                cache.set_user(user.into()).await?;
            }
        }
        Event::GuildScheduledEventDelete(event) => {
            if let Some(user) = event.creator.clone() {
                cache.set_user(user.into()).await?;
            }
        }
        Event::GuildScheduledEventUpdate(event) => {
            if let Some(user) = event.creator.clone() {
                cache.set_user(user.into()).await?;
            }
        }
        Event::GuildScheduledEventUserAdd(_) => {}
        Event::GuildScheduledEventUserRemove(_) => {}
        Event::GuildStickersUpdate(_) => {}
        Event::GuildUpdate(_) => {}
        Event::IntegrationCreate(_) => {}
        Event::IntegrationDelete(_) => {}
        Event::IntegrationUpdate(_) => {}
        Event::InteractionCreate(event) => {
            if let Some(user) = event.author().cloned() {
                cache.set_user(user.into()).await?;
            }

            // TODO: Update with partial users from `.resolved`.
        }
        Event::InviteCreate(event) => {
            if let Some(user) = event.inviter {
                cache.set_user(user.into()).await?;
            }
        }
        Event::InviteDelete(_) => {}
        Event::MemberAdd(event) => {
            cache.set_user(event.user.clone().into()).await?;
        }
        Event::MemberChunk(event) => {
            for member in event.members {
                cache.set_user(member.user.clone().into()).await?;
            }
        }
        Event::MemberRemove(event) => {
            cache.set_user(event.user.clone().into()).await?;
        }
        Event::MemberUpdate(event) => {
            cache.set_user(event.user.clone().into()).await?;
        }
        Event::MessageCreate(event) => {
            cache.set_user(event.author.clone().into()).await?;

            // TODO: Partially update from mention data.
        }
        Event::MessageDelete(_) => {}
        Event::MessageDeleteBulk(_) => {}
        Event::MessagePollVoteAdd(_) => {}
        Event::MessagePollVoteRemove(_) => {}
        Event::MessageUpdate(event) => {
            cache.set_user(event.author.clone().into()).await?;
        }
        Event::PresenceUpdate(event) => {
            if let UserOrId::User(user) = &event.user {
                cache.set_user(user.clone().into()).await?;
            }
        }
        Event::RateLimited(_) => {}
        Event::ReactionAdd(_) => {}
        Event::ReactionRemove(_) => {}
        Event::ReactionRemoveAll(_) => {}
        Event::ReactionRemoveEmoji(_) => {}
        Event::Ready(event) => {
            cache.set_user(event.user.into()).await?;
        }
        Event::Resumed => {}
        Event::RoleCreate(_) => {}
        Event::RoleDelete(_) => {}
        Event::RoleUpdate(_) => {}
        Event::StageInstanceCreate(_) => {}
        Event::StageInstanceDelete(_) => {}
        Event::StageInstanceUpdate(_) => {}
        Event::ThreadCreate(_) => {}
        Event::ThreadDelete(_) => {}
        Event::ThreadListSync(_) => {}
        Event::ThreadMemberUpdate(_) => {}
        Event::ThreadMembersUpdate(_) => {}
        Event::ThreadUpdate(_) => {}
        Event::TypingStart(_) => {}
        Event::UnavailableGuild(_) => {}
        Event::UserUpdate(event) => {
            cache.set_user(event.0.into()).await?;
        }
        Event::VoiceServerUpdate(_) => {}
        Event::VoiceStateUpdate(_) => {}
        Event::WebhooksUpdate(_) => {}
    }

    Ok(())
}
