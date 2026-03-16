use std::sync::Arc;

use twilight_gateway::Event;
use twilight_model::application::interaction::InteractionData;
use twilight_model::gateway::payload::incoming::InteractionCreate;

use crate::commands::CommandNode;
use crate::commands::slash::context::SlashContext;
use crate::commands::slash::{SlashCommand, SlashCommandGroup};
use crate::errors::{
    self, DyncordError, ErrorContext, ErrorHandlerWithoutType, ErrorOriginalContext,
};
use crate::events::EventContext;
use crate::state::StateBound;

pub async fn route_slash_command<State>(event_ctx: EventContext<State, InteractionCreate>)
where
    State: StateBound,
{
    if let Some(data) = &event_ctx.event.data
        && let InteractionData::ApplicationCommand(data) = data
    {
        for node in &*event_ctx.handle.commands {
            let command = match node {
                CommandNode::SlashCommand(command) => {
                    match_command(&event_ctx, &data.name, command)
                }
                CommandNode::SlashCommandGroup(group) => match_group(&event_ctx, &data.name, group),
                _ => {
                    continue;
                }
            };

            if let Some((command, mut error_handlers)) = command {
                let command_ctx = SlashContext {
                    state: event_ctx.state.clone(),
                    event: event_ctx.event.clone(),
                    event_data: (**data).clone(),
                    handle: event_ctx.handle.clone(),
                    command: command.clone(),
                };

                let result = command.handler.run(command_ctx.clone()).await;

                if let Err(error) = result {
                    let error_ctx = ErrorContext {
                        event: Event::InteractionCreate(Box::new(event_ctx.event)),
                        handle: event_ctx.handle,
                        state: event_ctx.state,
                        original: ErrorOriginalContext::SlashContext(Box::new(command_ctx)),
                    };

                    error_handlers.push(error_ctx.handle.on_errors.clone());

                    errors::handle(error_ctx, DyncordError::Command(error), &error_handlers).await;
                }

                break;
            }
        }
    }
}

type ErrorHandlers<State> = Vec<Vec<Arc<dyn ErrorHandlerWithoutType<State>>>>;

fn match_command<'a, State>(
    _ctx: &'a EventContext<State, InteractionCreate>,
    name: &str,
    command: &'a SlashCommand<State>,
) -> Option<(&'a SlashCommand<State>, ErrorHandlers<State>)>
where
    State: StateBound,
{
    if command.name == name {
        return Some((command, vec![command.on_errors.clone()]));
    }

    None
}

fn match_group<'a, State>(
    ctx: &'a EventContext<State, InteractionCreate>,
    parts: &str,
    group: &'a SlashCommandGroup<State>,
) -> Option<(&'a SlashCommand<State>, ErrorHandlers<State>)>
where
    State: StateBound,
{
    for node in &group.children {
        let command = match node {
            CommandNode::SlashCommand(subcommand) => match_command(ctx, parts, subcommand),
            CommandNode::SlashCommandGroup(subgroup) => match_group(ctx, parts, subgroup),
            _ => None,
        };

        if let Some((command, mut error_handlers)) = command {
            error_handlers.push(group.on_errors.clone());

            return Some((command, error_handlers));
        }
    }

    None
}
