use std::env;

use dyncord::Bot;
use dyncord::commands::Command;
use dyncord::commands::slash::context::SlashContext;
use dyncord::errors::{DyncordError, ErrorContext, ErrorHandlerError};

#[tokio::main]
async fn main() {
    let bot = Bot::new(())
        .with_prefix(".")
        .command(Command::slash("hello", hello).on_error(on_error_notify_user));

    bot.run(env::var("TOKEN").unwrap()).await.unwrap();
}

async fn hello(ctx: SlashContext, name: String) {
    ctx.respond(format!("Hello, {name}!")).await.unwrap();
}

async fn on_error_notify_user(
    ctx: ErrorContext,
    _error: DyncordError,
) -> Result<(), ErrorHandlerError> {
    ctx.send("Oh no! An error occurred.").await?;

    Ok(())
}
