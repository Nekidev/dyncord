//! A few common built-in utilities, like a help command.
//!
//! # Help Command
//!
//! The help command is a common command that displays a list of the bot's commands when called.
//! The built-in help command is implemented in the `help_command` function, and you can add it
//! normally to your bot's commands like so:
//!
//! ```rust
//! let bot = Bot::new(())
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .with_prefix("!")
//!     .command(Command::build("help", builtin::help_command));
//! ```
//!
//! Try calling `!help` in your server to see the list of commands, and `!help <command>` to see
//! the details of a specific command. You can also add summaries and descriptions to your commands
//! to make the help command more informative. For example:
//!
//! ```rust
//! let bot = Bot::new(())
//!     .intents(Intents::GUILD_MESSAGES)
//!     .intents(Intents::MESSAGE_CONTENT)
//!     .with_prefix("!")
//!     .command(Command::build("help", builtin::help_command).summary("Displays this message."))
//!     .command(Command::build("hello", dummy_command).summary("Says hi back."));
//!
//! async fn dummy_command(_ctx: CommandContext) {}
//! ```
//! 
//! Check out the [example](../source/examples/005_builtin_help.rs) for a more complete example of
//! how to use the help command.

use crate::commands::context::CommandContext;
use crate::commands::{self, Command, CommandGroup};
use crate::state::StateBound;

/// A help command handler, which displays a list of the bot's commands when called.
pub async fn help_command<State>(ctx: CommandContext<State>, command_name: Option<String>)
where
    State: StateBound,
{
    if let Some(command_name) = command_name {
        for command in commands::flatten(&ctx.handle.commands) {
            if command.identifiers().contains(&command_name) {
                ctx.send(display_command_help(&ctx.command_prefix, command))
                    .await
                    .unwrap();
                return;
            }
        }

        ctx.send("No command found with that name.").await.unwrap();
    } else {
        ctx.send(display_general_help(&ctx)).await.unwrap();
    }
}

fn display_general_help<State>(ctx: &CommandContext<State>) -> String
where
    State: StateBound,
{
    let bot_commands = commands::get_commands(&ctx.handle.commands);
    let bot_groups = commands::get_groups(&ctx.handle.commands);

    let mut result = String::new();

    result.push_str("```\n");

    // Crate name, version, and description
    result.push_str(&format!(
        "{} v{}\n\n{}\n",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION")
    ));

    for command in bot_commands {
        result.push_str(&display_command(&ctx.command_prefix, command, 0));
    }

    for group in bot_groups {
        result.push_str(&format!(
            "\n\n{}",
            display_group(&ctx.command_prefix, group, 0)
        ));
    }

    result.push_str("\n```");

    result
}

fn display_command<State>(prefix: &str, command: &Command<State>, indentation: usize) -> String
where
    State: StateBound,
{
    let mut result = format!("\n{}{}{}", " ".repeat(indentation), prefix, command.name);

    if let Some(summary) = &command.summary {
        result.push_str(&format!(" - {}", summary));
    }

    result
}

fn display_group<State>(prefix: &str, group: &CommandGroup<State>, indentation: usize) -> String
where
    State: StateBound,
{
    let mut result = String::new();

    let mut header = format!("{}{}", " ".repeat(indentation), group.name);

    if let Some(summary) = &group.summary {
        header.push_str(&format!(" - {}", summary));
    }

    header.push(':');

    result.push_str(&header);

    let bot_commands = commands::get_commands(&group.children);
    let bot_groups = commands::get_groups(&group.children);

    for command in bot_commands {
        result.push_str(&display_command(prefix, command, indentation + 2));
    }

    for group in bot_groups {
        result.push_str(&format!(
            "\n\n{}",
            display_group(prefix, group, indentation + 2)
        ));
    }

    result
}

fn display_command_help<State>(prefix: &str, command: &Command<State>) -> String
where
    State: StateBound,
{
    let mut result = String::from("```\n");

    result.push_str(&format!("{}{}", prefix, command.name));

    if let Some(summary) = &command.summary {
        result.push_str(&format!("\n\n{}", summary));
    }

    if let Some(description) = &command.description {
        result.push_str(&format!("\n\n{}", description));
    }

    result.push_str("\n```");

    result
}
