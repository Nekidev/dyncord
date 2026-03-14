//! Event handling system for Discord gateway events.
//!
//! Handling events in Dyncord is similar to implementing command handlers. You define a function
//! that takes [`EventContext<State, Event>`] as its only argument.
//!
//! For example, to make an event handler that echoes every message the bot receives, you'd do:
//!
//! ```
//! async fn on_message(ctx: EventContext<(), MessageCreate>) {
//!     if ctx.event.author.bot {
//!         return;
//!     }
//!
//!     ctx.handle.send(ctx.event.channel_id, &ctx.event.content).await.unwrap();
//! }
//! ```
//!
//! To add it to the bot, create an event handler and pass it to the bot using the
//! [`on_event`](crate::Bot::on_event) method, like so:
//!
//! ```
//! let bot = Bot::new(()).on_event(On::message_create(on_message));
//! ```
//!
//! Don't forget to add the intent corresponding to the event you want to handle! For example, for
//! `MessageCreate`, you need the
//! [`Intents::GUILD_MESSAGES`](twilight_gateway::Intents::GUILD_MESSAGES) and/or
//! [`Intents::DIRECT_MESSAGES`](twilight_gateway::Intents::DIRECT_MESSAGES) intents, plus the
//! [`Intents::MESSAGE_CONTENT`](twilight_gateway::Intents::MESSAGE_CONTENT) intent to be able to
//! read and echo the message content.
//!
//! ```
//! let bot = Bot::new(())
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .on_event(On::message_create(on_message));
//! ```
//!
//! To handle all incoming events, you can use the [`On::event`] handler, which will receive every
//! event as an [`Event`]. For example:
//!
//! ```
//! let bot = Bot::new(())
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .on_event(On::event(on_event));
//!
//! async fn on_event(ctx: EventContext<(), Event>) {
//!     // Handle the event.
//! }
//! ```

use std::sync::Arc;

pub use twilight_gateway::Event;
pub use twilight_model::gateway::payload::incoming::{
    AutoModerationActionExecution, AutoModerationRuleCreate, AutoModerationRuleDelete,
    AutoModerationRuleUpdate, BanAdd, BanRemove, ChannelCreate, ChannelDelete, ChannelPinsUpdate,
    ChannelUpdate, CommandPermissionsUpdate, EntitlementCreate, EntitlementDelete,
    EntitlementUpdate, GuildAuditLogEntryCreate, GuildCreate, GuildDelete, GuildEmojisUpdate,
    GuildIntegrationsUpdate, GuildScheduledEventCreate, GuildScheduledEventDelete,
    GuildScheduledEventUpdate, GuildScheduledEventUserAdd, GuildScheduledEventUserRemove,
    GuildStickersUpdate, GuildUpdate, Hello, IntegrationCreate, IntegrationDelete,
    IntegrationUpdate, InteractionCreate, InviteCreate, InviteDelete, MemberAdd, MemberChunk,
    MemberRemove, MemberUpdate, MessageCreate, MessageDelete, MessageDeleteBulk,
    MessagePollVoteAdd, MessagePollVoteRemove, MessageUpdate, PresenceUpdate, RateLimited,
    ReactionAdd, ReactionRemove, ReactionRemoveAll, ReactionRemoveEmoji, Ready, RoleCreate,
    RoleDelete, RoleUpdate, StageInstanceCreate, StageInstanceDelete, StageInstanceUpdate,
    ThreadCreate, ThreadDelete, ThreadListSync, ThreadMemberUpdate, ThreadMembersUpdate,
    ThreadUpdate, TypingStart, UnavailableGuild, UserUpdate, VoiceServerUpdate, VoiceStateUpdate,
    WebhooksUpdate,
};

use crate::handle::Handle;
use crate::state::StateBound;
use crate::utils::DynFuture;

#[derive(Clone)]
pub struct EventContext<State, E>
where
    State: StateBound,
{
    /// Your bot's state.
    pub state: State,

    /// A handle to the bot, which can be used to interact with the bot's internal state.
    pub handle: Handle<State>,

    /// The event that triggered the execution of this command.
    pub event: E,
}

pub trait EventHandler<State, E>
where
    State: StateBound,
{
    fn handle(&self, ctx: EventContext<State, E>) -> DynFuture<'static, ()>;
}

/// Implements [`EventHandler`] for any function `Fn(EventContext<State, E>) -> Fut`.
macro_rules! impl_event_handler_for {
    ($($event:ty),* $(,)?) => {
        $(
            impl<State, F, Fut> EventHandler<State, $event> for F
            where
                State: StateBound,
                F: Fn(EventContext<State, $event>) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = ()> + Send + 'static,
            {
                fn handle(&self, ctx: EventContext<State, $event>) -> DynFuture<'static, ()> {
                    Box::pin(self(ctx))
                }
            }
        )*
    };
}

impl_event_handler_for!(
    Event,
    AutoModerationActionExecution,
    AutoModerationRuleCreate,
    AutoModerationRuleDelete,
    AutoModerationRuleUpdate,
    BanAdd,
    BanRemove,
    ChannelCreate,
    ChannelDelete,
    ChannelUpdate,
    ChannelPinsUpdate,
    CommandPermissionsUpdate,
    EntitlementCreate,
    EntitlementDelete,
    EntitlementUpdate,
    GuildAuditLogEntryCreate,
    GuildCreate,
    GuildUpdate,
    GuildDelete,
    GuildEmojisUpdate,
    GuildIntegrationsUpdate,
    GuildScheduledEventCreate,
    GuildScheduledEventDelete,
    GuildScheduledEventUpdate,
    GuildScheduledEventUserAdd,
    GuildScheduledEventUserRemove,
    GuildStickersUpdate,
    IntegrationCreate,
    IntegrationUpdate,
    IntegrationDelete,
    InteractionCreate,
    InviteCreate,
    InviteDelete,
    MemberAdd,
    MemberRemove,
    MemberUpdate,
    MemberChunk,
    MessageCreate,
    MessageUpdate,
    MessageDelete,
    MessageDeleteBulk,
    MessagePollVoteAdd,
    MessagePollVoteRemove,
    PresenceUpdate,
    RateLimited,
    ReactionAdd,
    ReactionRemove,
    ReactionRemoveAll,
    ReactionRemoveEmoji,
    Ready,
    RoleCreate,
    RoleDelete,
    RoleUpdate,
    StageInstanceCreate,
    StageInstanceUpdate,
    StageInstanceDelete,
    ThreadCreate,
    ThreadUpdate,
    ThreadDelete,
    ThreadListSync,
    ThreadMemberUpdate,
    ThreadMembersUpdate,
    TypingStart,
    UnavailableGuild,
    UserUpdate,
    VoiceServerUpdate,
    VoiceStateUpdate,
    WebhooksUpdate,
);

pub struct GatewayClose;
pub struct GatewayHeartbeat;
pub struct GatewayHeartbeatAck;
pub struct GatewayInvalidateSession;
pub struct GatewayReconnect;
pub struct Resumed;

#[derive(Clone)]
pub enum OnEvent<State>
where
    State: StateBound,
{
    All(Arc<dyn EventHandler<State, Event>>),
    AutoModerationActionExecution(Arc<dyn EventHandler<State, AutoModerationActionExecution>>),
    AutoModerationRuleCreate(Arc<dyn EventHandler<State, AutoModerationRuleCreate>>),
    AutoModerationRuleDelete(Arc<dyn EventHandler<State, AutoModerationRuleDelete>>),
    AutoModerationRuleUpdate(Arc<dyn EventHandler<State, AutoModerationRuleUpdate>>),
    BanAdd(Arc<dyn EventHandler<State, BanAdd>>),
    BanRemove(Arc<dyn EventHandler<State, BanRemove>>),
    ChannelCreate(Arc<dyn EventHandler<State, ChannelCreate>>),
    ChannelDelete(Arc<dyn EventHandler<State, ChannelDelete>>),
    ChannelPinsUpdate(Arc<dyn EventHandler<State, ChannelPinsUpdate>>),
    ChannelUpdate(Arc<dyn EventHandler<State, ChannelUpdate>>),
    CommandPermissionsUpdate(Arc<dyn EventHandler<State, CommandPermissionsUpdate>>),
    EntitlementCreate(Arc<dyn EventHandler<State, EntitlementCreate>>),
    EntitlementDelete(Arc<dyn EventHandler<State, EntitlementDelete>>),
    EntitlementUpdate(Arc<dyn EventHandler<State, EntitlementUpdate>>),
    GatewayClose(Arc<dyn EventHandler<State, GatewayClose>>),
    GatewayHeartbeat(Arc<dyn EventHandler<State, GatewayHeartbeat>>),
    GatewayHeartbeatAck(Arc<dyn EventHandler<State, GatewayHeartbeatAck>>),
    GatewayHello(Arc<dyn EventHandler<State, Hello>>),
    GatewayInvalidateSession(Arc<dyn EventHandler<State, GatewayInvalidateSession>>),
    GatewayReconnect(Arc<dyn EventHandler<State, GatewayReconnect>>),
    GuildAuditLogEntryCreate(Arc<dyn EventHandler<State, GuildAuditLogEntryCreate>>),
    GuildCreate(Arc<dyn EventHandler<State, GuildCreate>>),
    GuildDelete(Arc<dyn EventHandler<State, GuildDelete>>),
    GuildEmojisUpdate(Arc<dyn EventHandler<State, GuildEmojisUpdate>>),
    GuildIntegrationsUpdate(Arc<dyn EventHandler<State, GuildIntegrationsUpdate>>),
    GuildScheduledEventCreate(Arc<dyn EventHandler<State, GuildScheduledEventCreate>>),
    GuildScheduledEventDelete(Arc<dyn EventHandler<State, GuildScheduledEventDelete>>),
    GuildScheduledEventUpdate(Arc<dyn EventHandler<State, GuildScheduledEventUpdate>>),
    GuildScheduledEventUserAdd(Arc<dyn EventHandler<State, GuildScheduledEventUserAdd>>),
    GuildScheduledEventUserRemove(Arc<dyn EventHandler<State, GuildScheduledEventUserRemove>>),
    GuildStickersUpdate(Arc<dyn EventHandler<State, GuildStickersUpdate>>),
    GuildUpdate(Arc<dyn EventHandler<State, GuildUpdate>>),
    IntegrationCreate(Arc<dyn EventHandler<State, IntegrationCreate>>),
    IntegrationDelete(Arc<dyn EventHandler<State, IntegrationDelete>>),
    IntegrationUpdate(Arc<dyn EventHandler<State, IntegrationUpdate>>),
    InteractionCreate(Arc<dyn EventHandler<State, InteractionCreate>>),
    InviteCreate(Arc<dyn EventHandler<State, InviteCreate>>),
    InviteDelete(Arc<dyn EventHandler<State, InviteDelete>>),
    MemberAdd(Arc<dyn EventHandler<State, MemberAdd>>),
    MemberChunk(Arc<dyn EventHandler<State, MemberChunk>>),
    MemberRemove(Arc<dyn EventHandler<State, MemberRemove>>),
    MemberUpdate(Arc<dyn EventHandler<State, MemberUpdate>>),
    MessageCreate(Arc<dyn EventHandler<State, MessageCreate>>),
    MessageDelete(Arc<dyn EventHandler<State, MessageDelete>>),
    MessageDeleteBulk(Arc<dyn EventHandler<State, MessageDeleteBulk>>),
    MessagePollVoteAdd(Arc<dyn EventHandler<State, MessagePollVoteAdd>>),
    MessagePollVoteRemove(Arc<dyn EventHandler<State, MessagePollVoteRemove>>),
    MessageUpdate(Arc<dyn EventHandler<State, MessageUpdate>>),
    PresenceUpdate(Arc<dyn EventHandler<State, PresenceUpdate>>),
    RateLimited(Arc<dyn EventHandler<State, RateLimited>>),
    ReactionAdd(Arc<dyn EventHandler<State, ReactionAdd>>),
    ReactionRemove(Arc<dyn EventHandler<State, ReactionRemove>>),
    ReactionRemoveAll(Arc<dyn EventHandler<State, ReactionRemoveAll>>),
    ReactionRemoveEmoji(Arc<dyn EventHandler<State, ReactionRemoveEmoji>>),
    Ready(Arc<dyn EventHandler<State, Ready>>),
    Resumed(Arc<dyn EventHandler<State, Resumed>>),
    RoleCreate(Arc<dyn EventHandler<State, RoleCreate>>),
    RoleDelete(Arc<dyn EventHandler<State, RoleDelete>>),
    RoleUpdate(Arc<dyn EventHandler<State, RoleUpdate>>),
    StageInstanceCreate(Arc<dyn EventHandler<State, StageInstanceCreate>>),
    StageInstanceDelete(Arc<dyn EventHandler<State, StageInstanceDelete>>),
    StageInstanceUpdate(Arc<dyn EventHandler<State, StageInstanceUpdate>>),
    ThreadCreate(Arc<dyn EventHandler<State, ThreadCreate>>),
    ThreadDelete(Arc<dyn EventHandler<State, ThreadDelete>>),
    ThreadListSync(Arc<dyn EventHandler<State, ThreadListSync>>),
    ThreadMemberUpdate(Arc<dyn EventHandler<State, ThreadMemberUpdate>>),
    ThreadMembersUpdate(Arc<dyn EventHandler<State, ThreadMembersUpdate>>),
    ThreadUpdate(Arc<dyn EventHandler<State, ThreadUpdate>>),
    TypingStart(Arc<dyn EventHandler<State, TypingStart>>),
    UnavailableGuild(Arc<dyn EventHandler<State, UnavailableGuild>>),
    UserUpdate(Arc<dyn EventHandler<State, UserUpdate>>),
    VoiceServerUpdate(Arc<dyn EventHandler<State, VoiceServerUpdate>>),
    VoiceStateUpdate(Arc<dyn EventHandler<State, VoiceStateUpdate>>),
    WebhooksUpdate(Arc<dyn EventHandler<State, WebhooksUpdate>>),
}

macro_rules! match_event_type(
    () => {};

    ($event:ident $handler:ident $state:ident $handle:ident $type:ident, $($rest:tt)*) => {
        if let Event::$type(event) = &$event && let Self::$type(handler) = $handler {
            let ctx = EventContext {
                state: $state,
                handle: $handle,
                event: event.clone(),
            };
            return Some(Box::pin(handler.handle(ctx)));
        }

        match_event_type!($($rest)*);
    };

    ($event:ident $handler:ident $state:ident $handle:ident $type:ident boxed, $($rest:tt)*) => {
        if let Event::$type(event) = &$event && let Self::$type(handler) = $handler {
            let ctx = EventContext {
                state: $state,
                handle: $handle,
                event: (**event).clone(),
            };
            return Some(Box::pin(handler.handle(ctx)));
        }

        match_event_type!($($rest)*);
    };

    ($event:ident $handler:ident $state:ident $handle:ident $type:ident placeholder, $($rest:tt)*) => {
        if let Event::$type = &$event && let Self::$type(handler) = $handler {
            let ctx = EventContext {
                state: $state,
                handle: $handle,
                event: $type,
            };
            return Some(Box::pin(handler.handle(ctx)));
        }

        match_event_type!($($rest)*);
    };

    ($event:ident $handler:ident $state:ident $handle:ident $type:ident discarded, $($rest:tt)*) => {
        if let Event::$type(_) = &$event && let Self::$type(handler) = $handler {
            let ctx = EventContext {
                state: $state,
                handle: $handle,
                event: $type,
            };
            return Some(Box::pin(handler.handle(ctx)));
        }

        match_event_type!($($rest)*);
    }
);

impl<State> OnEvent<State>
where
    State: StateBound,
{
    pub fn handle(
        &self,
        handle: Handle<State>,
        state: State,
        event: Event,
    ) -> Option<DynFuture<'static, ()>> {
        if let Self::All(handler) = self {
            let ctx = EventContext {
                state,
                handle,
                event: event.clone(),
            };
            return Some(Box::pin(handler.handle(ctx)));
        }

        match_event_type!(
            event self state handle AutoModerationActionExecution,
            event self state handle AutoModerationRuleCreate,
            event self state handle AutoModerationRuleDelete,
            event self state handle AutoModerationRuleUpdate,
            event self state handle BanAdd,
            event self state handle BanRemove,
            event self state handle ChannelCreate boxed,
            event self state handle ChannelDelete boxed,
            event self state handle ChannelUpdate boxed,
            event self state handle ChannelPinsUpdate,
            event self state handle CommandPermissionsUpdate,
            event self state handle EntitlementCreate,
            event self state handle EntitlementDelete,
            event self state handle EntitlementUpdate,
            event self state handle GuildAuditLogEntryCreate boxed,
            event self state handle GuildCreate boxed,
            event self state handle GuildUpdate boxed,
            event self state handle GuildDelete,
            event self state handle GuildEmojisUpdate,
            event self state handle GuildIntegrationsUpdate,
            event self state handle GuildScheduledEventCreate boxed,
            event self state handle GuildScheduledEventDelete boxed,
            event self state handle GuildScheduledEventUpdate boxed,
            event self state handle GuildScheduledEventUserAdd,
            event self state handle GuildScheduledEventUserRemove,
            event self state handle GuildStickersUpdate,
            event self state handle IntegrationCreate boxed,
            event self state handle IntegrationUpdate boxed,
            event self state handle IntegrationDelete,
            event self state handle InteractionCreate boxed,
            event self state handle InviteCreate boxed,
            event self state handle InviteDelete,
            event self state handle MemberAdd boxed,
            event self state handle MemberRemove,
            event self state handle MemberUpdate boxed,
            event self state handle MemberChunk,
            event self state handle MessageCreate boxed,
            event self state handle MessageUpdate boxed,
            event self state handle MessageDelete,
            event self state handle MessageDeleteBulk,
            event self state handle MessagePollVoteAdd,
            event self state handle MessagePollVoteRemove,
            event self state handle PresenceUpdate boxed,
            event self state handle RateLimited,
            event self state handle ReactionAdd boxed,
            event self state handle ReactionRemove boxed,
            event self state handle ReactionRemoveAll,
            event self state handle ReactionRemoveEmoji,
            event self state handle Ready,
            event self state handle RoleCreate,
            event self state handle RoleDelete,
            event self state handle RoleUpdate,
            event self state handle StageInstanceCreate,
            event self state handle StageInstanceUpdate,
            event self state handle StageInstanceDelete,
            event self state handle ThreadCreate boxed,
            event self state handle ThreadUpdate boxed,
            event self state handle ThreadDelete,
            event self state handle ThreadListSync,
            event self state handle ThreadMemberUpdate boxed,
            event self state handle ThreadMembersUpdate,
            event self state handle TypingStart boxed,
            event self state handle UnavailableGuild,
            event self state handle UserUpdate,
            event self state handle VoiceServerUpdate,
            event self state handle VoiceStateUpdate boxed,
            event self state handle WebhooksUpdate,

            event self state handle GatewayClose discarded,
            event self state handle GatewayHeartbeat placeholder,
            event self state handle GatewayHeartbeatAck placeholder,
            event self state handle GatewayHello,
            event self state handle GatewayInvalidateSession discarded,
            event self state handle GatewayReconnect placeholder,
            event self state handle Resumed placeholder,
        );

        None
    }
}

/// A helper struct to create event handlers more easily.
///
/// Each method corresponds to a different event type, and takes a function that will be called
/// when the event is received. For example, to create a handler for the `MessageCreate` event, you
/// can do:
///
/// ```
/// let bot = Bot::new(())
///     .on_event(On::message_create(on_message));
/// ```
///
/// This struct's methods are just proxies to creating [`OnEvent`] more prettily.
pub struct On;

macro_rules! impl_on {
    ($event:ident $func:ident) => {
        pub fn $func<F, State>(handler: F) -> OnEvent<State>
        where
            F: EventHandler<State, $event> + 'static,
            State: StateBound,
        {
            OnEvent::$event(Arc::new(handler))
        }
    };

    ($event:ident $func:ident ($inner:ident)) => {
        pub fn $func<F, State>(handler: F) -> OnEvent<State>
        where
            F: EventHandler<State, $inner> + 'static,
            State: StateBound,
        {
            OnEvent::$event(Arc::new(handler))
        }
    };
}

impl On {
    impl_on!(All event (Event));
    impl_on!(AutoModerationActionExecution auto_moderation_action_execution);
    impl_on!(AutoModerationRuleCreate auto_moderation_rule_create);
    impl_on!(AutoModerationRuleDelete auto_moderation_rule_delete);
    impl_on!(AutoModerationRuleUpdate auto_moderation_rule_update);
    impl_on!(BanAdd ban_add);
    impl_on!(BanRemove ban_remove);
    impl_on!(ChannelCreate channel_create);
    impl_on!(ChannelDelete channel_delete);
    impl_on!(ChannelPinsUpdate channel_pins_update);
    impl_on!(ChannelUpdate channel_update);
    impl_on!(CommandPermissionsUpdate command_permissions_update);
    impl_on!(EntitlementCreate entitlement_create);
    impl_on!(EntitlementDelete entitlement_delete);
    impl_on!(EntitlementUpdate entitlement_update);
    impl_on!(GatewayClose gateway_close);
    impl_on!(GatewayHeartbeat gateway_heartbeat);
    impl_on!(GatewayHeartbeatAck gateway_heartbeat_ack);
    impl_on!(GatewayHello gateway_hello (Hello));
    impl_on!(GatewayInvalidateSession gateway_invalidate_session);
    impl_on!(GatewayReconnect gateway_reconnect);
    impl_on!(GuildAuditLogEntryCreate guild_audit_log_entry_create);
    impl_on!(GuildCreate guild_create);
    impl_on!(GuildDelete guild_delete);
    impl_on!(GuildEmojisUpdate guild_emojis_update);
    impl_on!(GuildIntegrationsUpdate guild_integrations_update);
    impl_on!(GuildScheduledEventCreate guild_scheduled_event_create);
    impl_on!(GuildScheduledEventDelete guild_scheduled_event_delete);
    impl_on!(GuildScheduledEventUpdate guild_scheduled_event_update);
    impl_on!(GuildScheduledEventUserAdd guild_scheduled_event_user_add);
    impl_on!(GuildScheduledEventUserRemove guild_scheduled_event_user_remove);
    impl_on!(GuildStickersUpdate guild_stickers_update);
    impl_on!(GuildUpdate guild_update);
    impl_on!(IntegrationCreate integration_create);
    impl_on!(IntegrationDelete integration_delete);
    impl_on!(IntegrationUpdate integration_update);
    impl_on!(InteractionCreate interaction_create);
    impl_on!(InviteCreate invite_create);
    impl_on!(InviteDelete invite_delete);
    impl_on!(MemberAdd member_add);
    impl_on!(MemberChunk member_chunk);
    impl_on!(MemberRemove member_remove);
    impl_on!(MemberUpdate member_update);
    impl_on!(MessageCreate message_create);
    impl_on!(MessageDelete message_delete);
    impl_on!(MessageDeleteBulk message_delete_bulk);
    impl_on!(MessagePollVoteAdd message_poll_vote_add);
    impl_on!(MessagePollVoteRemove message_poll_vote_remove);
    impl_on!(MessageUpdate message_update);
    impl_on!(PresenceUpdate presence_update);
    impl_on!(RateLimited rate_limited);
    impl_on!(ReactionAdd reaction_add);
    impl_on!(ReactionRemove reaction_remove);
    impl_on!(ReactionRemoveAll reaction_remove_all);
    impl_on!(ReactionRemoveEmoji reaction_remove_emoji);
    impl_on!(Ready ready);
    impl_on!(Resumed resumed);
    impl_on!(RoleCreate role_create);
    impl_on!(RoleDelete role_delete);
    impl_on!(RoleUpdate role_update);
    impl_on!(StageInstanceCreate stage_instance_create);
    impl_on!(StageInstanceDelete stage_instance_delete);
    impl_on!(StageInstanceUpdate stage_instance_update);
    impl_on!(ThreadCreate thread_create);
    impl_on!(ThreadDelete thread_delete);
    impl_on!(ThreadListSync thread_list_sync);
    impl_on!(ThreadMemberUpdate thread_member_update);
    impl_on!(ThreadMembersUpdate thread_members_update);
    impl_on!(ThreadUpdate thread_update);
    impl_on!(TypingStart typing_start);
    impl_on!(UnavailableGuild unavailable_guild);
    impl_on!(UserUpdate user_update);
    impl_on!(VoiceServerUpdate voice_server_update);
    impl_on!(VoiceStateUpdate voice_state_update);
    impl_on!(WebhooksUpdate webhooks_update);
}
