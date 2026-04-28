use std::sync::Arc;

use twilight_gateway::Event;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::commands::CommandNode;
use crate::commands::errors::CommandError;
use crate::commands::prefixed::context::PrefixedContext;
use crate::commands::prefixed::parsing::{self, CommandParts};
use crate::commands::prefixed::prefixes::PrefixesContext;
use crate::commands::prefixed::{PrefixedCommand, PrefixedCommandGroup};
use crate::errors::{
    self, DyncordError, ErrorContext, ErrorHandlerWithoutType, ErrorOriginalContext,
};
use crate::events::EventContext;
use crate::state::StateBound;

/// Handles the invokation of message commands.
///
/// When a [`MessageCreate`] event is received, this function checks if the message starts with any
/// of the bot's registered commands and invokes it.
pub(crate) async fn route_prefixed_command<State>(event_ctx: EventContext<State, MessageCreate>)
where
    State: StateBound,
{
    if let Some(prefixes) = &event_ctx.handle.prefixes {
        let prefixes_ctx = PrefixesContext {
            state: event_ctx.state.clone(),
            event: event_ctx.event.clone(),
        };

        let prefixes = match prefixes.get_prefixes(prefixes_ctx.clone()).await {
            Ok(prefixes) => prefixes,
            Err(error) => {
                let error_ctx = ErrorContext {
                    event: Event::MessageCreate(Box::new(event_ctx.event)),
                    handle: event_ctx.handle.clone(),
                    state: event_ctx.state,
                    original: ErrorOriginalContext::PrefixesContext(Box::new(prefixes_ctx)),
                };

                errors::handle(
                    error_ctx,
                    DyncordError::Command(CommandError::Prefixes(error)),
                    &[event_ctx.handle.on_errors],
                )
                .await;

                return;
            }
        };

        'prefixes: for prefix in prefixes {
            let Some(parts) = parsing::parse(&prefix, &event_ctx.event.content) else {
                continue;
            };

            for node in &*event_ctx.handle.commands {
                let command = match node {
                    CommandNode::PrefixedCommand(command) => {
                        match_command(&event_ctx, parts, command)
                    }
                    CommandNode::PrefixedCommandGroup(group) => {
                        match_group(&event_ctx, parts, group)
                    }
                    _ => {
                        continue;
                    }
                };

                if let Some((command, mut error_handlers)) = command {
                    let command_identifier = parts.command_name.to_string();
                    let command_prefix = parts.prefix.to_string();
                    let command_args = parts.command_args.to_string();

                    let command_ctx = PrefixedContext {
                        event: event_ctx.event.clone(),
                        state: event_ctx.state.clone(),
                        handle: event_ctx.handle.clone(),
                        command_identifier,
                        command_prefix,
                        command_args,
                    };

                    let result = command.run(command_ctx.clone(), parts.command_args).await;

                    if let Err(error) = result {
                        let error_ctx = ErrorContext {
                            event: Event::MessageCreate(Box::new(event_ctx.event)),
                            handle: event_ctx.handle,
                            state: event_ctx.state,
                            original: ErrorOriginalContext::PrefixedContext(Box::new(command_ctx)),
                        };

                        error_handlers.push(error_ctx.handle.on_errors.clone());

                        errors::handle(error_ctx, DyncordError::Command(error), &error_handlers)
                            .await;
                    }

                    break 'prefixes;
                }
            }
        }
    }
}

type ErrorHandlers<State> = Vec<Vec<Arc<dyn ErrorHandlerWithoutType<State>>>>;

fn match_command<'a, State>(
    _ctx: &'a EventContext<State, MessageCreate>,
    parts: CommandParts<'a>,
    command: &'a PrefixedCommand<State>,
) -> Option<(&'a PrefixedCommand<State>, ErrorHandlers<State>)>
where
    State: StateBound,
{
    if command
        .identifiers()
        .contains(&parts.command_name.to_string())
    {
        return Some((command, vec![command.on_errors.clone()]));
    }

    None
}

fn match_group<'a, State>(
    ctx: &'a EventContext<State, MessageCreate>,
    parts: CommandParts<'a>,
    group: &'a PrefixedCommandGroup<State>,
) -> Option<(&'a PrefixedCommand<State>, ErrorHandlers<State>)>
where
    State: StateBound,
{
    for node in &group.children {
        let command = match node {
            CommandNode::PrefixedCommand(subcommand) => match_command(ctx, parts, subcommand),
            CommandNode::PrefixedCommandGroup(subgroup) => match_group(ctx, parts, subgroup),
            _ => None,
        };

        if let Some((command, mut error_handlers)) = command {
            error_handlers.push(group.on_errors.clone());

            return Some((command, error_handlers));
        }
    }

    None
}
