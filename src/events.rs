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

use std::error::Error;
use std::marker::PhantomData;
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

use crate::errors::{ErrorHandler, ErrorHandlerWithoutType, ErrorHandlerWrapper};
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

/// The result type to which all event handler function return values are converted.
pub type EventResult = Result<(), Arc<dyn Error + Send + Sync>>;

pub trait IntoEventResult {
    fn into_event_result(self) -> EventResult;
}

impl IntoEventResult for () {
    fn into_event_result(self) -> EventResult {
        Ok(())
    }
}

impl<T, E> IntoEventResult for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    fn into_event_result(self) -> EventResult {
        match self {
            Ok(_) => Ok(()),
            Err(error) => Err(Arc::new(error)),
        }
    }
}

#[derive(Clone)]
pub(crate) struct EventHandler<State>
where
    State: StateBound,
{
    pub handler: Arc<dyn EventHandlerHandlerWithoutArgs<State> + 'static>,
    pub on_errors: Vec<Arc<dyn ErrorHandlerWithoutType<State>>>,
}

pub struct EventHandlerBuilder<State>
where
    State: StateBound,
{
    handler: Arc<dyn EventHandlerHandlerWithoutArgs<State> + 'static>,
    on_errors: Vec<Arc<dyn ErrorHandlerWithoutType<State>>>,
}

impl<State> EventHandlerBuilder<State>
where
    State: StateBound,
{
    pub fn on_error<Error>(mut self, handler: impl ErrorHandler<State, Error> + 'static) -> Self
    where
        Error: Send + Sync + 'static,
    {
        self.on_errors
            .push(Arc::new(ErrorHandlerWrapper::new(handler)));
        self
    }

    pub(crate) fn build(self) -> EventHandler<State> {
        EventHandler {
            handler: self.handler,
            on_errors: self.on_errors,
        }
    }
}

/// Trait implemented by all event handler functions.
pub trait EventHandlerHandler<State, E>: Send + Sync
where
    State: StateBound,
{
    fn handle(&self, ctx: EventContext<State, E>) -> DynFuture<'_, EventResult>;
}

/// Implements [`EventHandler`] for any function `Fn(EventContext<State, E>) -> Fut`.
macro_rules! impl_event_handler_for {
    ($($event:ty),* $(,)?) => {
        $(
            impl<State, F, Fut, Res> EventHandlerHandler<State, $event> for F
            where
                State: StateBound,
                F: Fn(EventContext<State, $event>) -> Fut + Send + Sync + 'static,
                Fut: Future<Output = Res> + Send + 'static,
                Res: IntoEventResult,
            {
                fn handle(&self, ctx: EventContext<State, $event>) -> DynFuture<'_, EventResult> {
                    Box::pin(async move { self(ctx).await.into_event_result() })
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

pub struct EventHandlerWrapper<F, Event> {
    func: F,
    _event: PhantomData<Event>,
}

impl<F, Event> EventHandlerWrapper<F, Event> {
    fn new(func: F) -> Self {
        EventHandlerWrapper {
            func,
            _event: PhantomData,
        }
    }
}

pub trait EventHandlerHandlerWithoutArgs<State>: Send + Sync
where
    State: StateBound,
{
    fn handle(&self, ctx: EventContext<State, Event>) -> Option<DynFuture<'_, EventResult>>;

    fn check_type(&self, event: &Event) -> bool;
}

macro_rules! impl_event_handler_handler_without_args {
    ($event:ident) => {
        impl<State, Func> EventHandlerHandlerWithoutArgs<State>
            for EventHandlerWrapper<Func, $event>
        where
            State: StateBound,
            Func: EventHandlerHandler<State, $event>,
        {
            fn handle(
                &self,
                ctx: EventContext<State, Event>,
            ) -> Option<DynFuture<'_, EventResult>> {
                if let Event::$event(event) = ctx.event {
                    let ctx = EventContext {
                        event: event.clone(),
                        handle: ctx.handle.clone(),
                        state: ctx.state.clone(),
                    };

                    return Some(self.func.handle(ctx));
                }

                None
            }

            fn check_type(&self, event: &Event) -> bool {
                matches!(event, Event::$event(_))
            }
        }
    };

    ($event:ident boxed) => {
        impl<State, Func> EventHandlerHandlerWithoutArgs<State>
            for EventHandlerWrapper<Func, $event>
        where
            State: StateBound,
            Func: EventHandlerHandler<State, $event>,
        {
            fn handle(
                &self,
                ctx: EventContext<State, Event>,
            ) -> Option<DynFuture<'_, EventResult>> {
                if let Event::$event(event) = ctx.event {
                    let ctx = EventContext {
                        event: (*event).clone(),
                        handle: ctx.handle.clone(),
                        state: ctx.state.clone(),
                    };

                    return Some(self.func.handle(ctx));
                }

                None
            }

            fn check_type(&self, event: &Event) -> bool {
                matches!(event, Event::$event(_))
            }
        }
    };

    ($event:ident discarded) => {
        impl<State, Func> EventHandlerHandlerWithoutArgs<State>
            for EventHandlerWrapper<Func, $event>
        where
            State: StateBound,
            Func: EventHandlerHandler<State, $event>,
        {
            fn handle(
                &self,
                ctx: EventContext<State, Event>,
            ) -> Option<DynFuture<'_, EventResult>> {
                if let Event::$event(_) = ctx.event {
                    let ctx = EventContext {
                        event: $event,
                        handle: ctx.handle.clone(),
                        state: ctx.state.clone(),
                    };

                    return Some(self.func.handle(ctx));
                }

                None
            }

            fn check_type(&self, event: &Event) -> bool {
                matches!(event, Event::$event(_))
            }
        }
    };

    ($event:ident placeholder) => {
        impl<State, Func> EventHandlerHandlerWithoutArgs<State>
            for EventHandlerWrapper<Func, $event>
        where
            State: StateBound,
            Func: EventHandlerHandler<State, $event>,
        {
            fn handle(
                &self,
                ctx: EventContext<State, Event>,
            ) -> Option<DynFuture<'_, EventResult>> {
                if let Event::$event = ctx.event {
                    let ctx = EventContext {
                        event: $event,
                        handle: ctx.handle.clone(),
                        state: ctx.state.clone(),
                    };

                    return Some(self.func.handle(ctx));
                }

                None
            }

            fn check_type(&self, event: &Event) -> bool {
                matches!(event, Event::$event)
            }
        }
    };

    ($event:ident where inner = $inner:ident) => {
        impl<State, Func> EventHandlerHandlerWithoutArgs<State>
            for EventHandlerWrapper<Func, $inner>
        where
            State: StateBound,
            Func: EventHandlerHandler<State, $inner>,
        {
            fn handle(
                &self,
                ctx: EventContext<State, Event>,
            ) -> Option<DynFuture<'_, EventResult>> {
                if let Event::$event(event) = ctx.event {
                    let ctx = EventContext {
                        event,
                        handle: ctx.handle.clone(),
                        state: ctx.state.clone(),
                    };

                    return Some(self.func.handle(ctx));
                }

                None
            }

            fn check_type(&self, event: &Event) -> bool {
                matches!(event, Event::$event(_))
            }
        }
    };
}

impl<State, Func> EventHandlerHandlerWithoutArgs<State> for EventHandlerWrapper<Func, Event>
where
    State: StateBound,
    Func: EventHandlerHandler<State, Event>,
{
    fn handle(&self, ctx: EventContext<State, Event>) -> Option<DynFuture<'_, EventResult>> {
        Some(self.func.handle(ctx))
    }

    fn check_type(&self, _: &Event) -> bool {
        true
    }
}

impl_event_handler_handler_without_args!(AutoModerationActionExecution);
impl_event_handler_handler_without_args!(AutoModerationRuleCreate);
impl_event_handler_handler_without_args!(AutoModerationRuleDelete);
impl_event_handler_handler_without_args!(AutoModerationRuleUpdate);
impl_event_handler_handler_without_args!(BanAdd);
impl_event_handler_handler_without_args!(BanRemove);
impl_event_handler_handler_without_args!(ChannelCreate boxed);
impl_event_handler_handler_without_args!(ChannelDelete boxed);
impl_event_handler_handler_without_args!(ChannelUpdate boxed);
impl_event_handler_handler_without_args!(ChannelPinsUpdate);
impl_event_handler_handler_without_args!(CommandPermissionsUpdate);
impl_event_handler_handler_without_args!(EntitlementCreate);
impl_event_handler_handler_without_args!(EntitlementDelete);
impl_event_handler_handler_without_args!(EntitlementUpdate);
impl_event_handler_handler_without_args!(GuildAuditLogEntryCreate boxed);
impl_event_handler_handler_without_args!(GuildCreate boxed);
impl_event_handler_handler_without_args!(GuildUpdate boxed);
impl_event_handler_handler_without_args!(GuildDelete);
impl_event_handler_handler_without_args!(GuildEmojisUpdate);
impl_event_handler_handler_without_args!(GuildIntegrationsUpdate);
impl_event_handler_handler_without_args!(GuildScheduledEventCreate boxed);
impl_event_handler_handler_without_args!(GuildScheduledEventDelete boxed);
impl_event_handler_handler_without_args!(GuildScheduledEventUpdate boxed);
impl_event_handler_handler_without_args!(GuildScheduledEventUserAdd);
impl_event_handler_handler_without_args!(GuildScheduledEventUserRemove);
impl_event_handler_handler_without_args!(GuildStickersUpdate);
impl_event_handler_handler_without_args!(IntegrationCreate boxed);
impl_event_handler_handler_without_args!(IntegrationUpdate boxed);
impl_event_handler_handler_without_args!(IntegrationDelete);
impl_event_handler_handler_without_args!(InteractionCreate boxed);
impl_event_handler_handler_without_args!(InviteCreate boxed);
impl_event_handler_handler_without_args!(InviteDelete);
impl_event_handler_handler_without_args!(MemberAdd boxed);
impl_event_handler_handler_without_args!(MemberRemove);
impl_event_handler_handler_without_args!(MemberUpdate boxed);
impl_event_handler_handler_without_args!(MemberChunk);
impl_event_handler_handler_without_args!(MessageCreate boxed);
impl_event_handler_handler_without_args!(MessageUpdate boxed);
impl_event_handler_handler_without_args!(MessageDelete);
impl_event_handler_handler_without_args!(MessageDeleteBulk);
impl_event_handler_handler_without_args!(MessagePollVoteAdd);
impl_event_handler_handler_without_args!(MessagePollVoteRemove);
impl_event_handler_handler_without_args!(PresenceUpdate boxed);
impl_event_handler_handler_without_args!(RateLimited);
impl_event_handler_handler_without_args!(ReactionAdd boxed);
impl_event_handler_handler_without_args!(ReactionRemove boxed);
impl_event_handler_handler_without_args!(ReactionRemoveAll);
impl_event_handler_handler_without_args!(ReactionRemoveEmoji);
impl_event_handler_handler_without_args!(Ready);
impl_event_handler_handler_without_args!(RoleCreate);
impl_event_handler_handler_without_args!(RoleDelete);
impl_event_handler_handler_without_args!(RoleUpdate);
impl_event_handler_handler_without_args!(StageInstanceCreate);
impl_event_handler_handler_without_args!(StageInstanceUpdate);
impl_event_handler_handler_without_args!(StageInstanceDelete);
impl_event_handler_handler_without_args!(ThreadCreate boxed);
impl_event_handler_handler_without_args!(ThreadUpdate boxed);
impl_event_handler_handler_without_args!(ThreadDelete);
impl_event_handler_handler_without_args!(ThreadListSync);
impl_event_handler_handler_without_args!(ThreadMemberUpdate boxed);
impl_event_handler_handler_without_args!(ThreadMembersUpdate);
impl_event_handler_handler_without_args!(TypingStart boxed);
impl_event_handler_handler_without_args!(UnavailableGuild);
impl_event_handler_handler_without_args!(UserUpdate);
impl_event_handler_handler_without_args!(VoiceServerUpdate);
impl_event_handler_handler_without_args!(VoiceStateUpdate boxed);
impl_event_handler_handler_without_args!(WebhooksUpdate);

impl_event_handler_handler_without_args!(GatewayClose discarded);
impl_event_handler_handler_without_args!(GatewayHeartbeat placeholder);
impl_event_handler_handler_without_args!(GatewayHeartbeatAck placeholder);
impl_event_handler_handler_without_args!(GatewayHello where inner = Hello);
impl_event_handler_handler_without_args!(GatewayInvalidateSession discarded);
impl_event_handler_handler_without_args!(GatewayReconnect placeholder);
impl_event_handler_handler_without_args!(Resumed placeholder);

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
        pub fn $func<F, State>(handler: F) -> EventHandlerBuilder<State>
        where
            F: EventHandlerHandler<State, $event> + 'static,
            State: StateBound,
        {
            EventHandlerBuilder {
                handler: Arc::new(EventHandlerWrapper::new(handler)),
                on_errors: Vec::new(),
            }
        }
    };

    ($event:ident $func:ident ($inner:ident)) => {
        pub fn $func<F, State>(handler: F) -> EventHandlerBuilder<State>
        where
            F: EventHandlerHandler<State, $inner> + 'static,
            State: StateBound,
        {
            EventHandlerBuilder {
                handler: Arc::new(EventHandlerWrapper::new(handler)),
                on_errors: Vec::new(),
            }
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
