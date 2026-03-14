use std::env;

use dyncord::Bot;
use dyncord::commands::slash::arguments::Argument;
use dyncord::commands::slash::context::SlashContext;
use dyncord::commands::{Command, CommandGroup};

#[tokio::main]
async fn main() {
    let bot = Bot::new(())
        .with_prefix(".")
        .command(
            Command::slash("hello", hello)
                .description("Says hi back.")
                .argument(
                    Argument::string("name")
                        .description("Your name, to sell it to *ahem* to say hi.")
                        .optional(),
                ),
        )
        .nest(CommandGroup::slash("hey"));

    bot.run(env::var("TOKEN").unwrap()).await.unwrap();
}

async fn hello(ctx: SlashContext, name: Option<String>) {
    ctx.respond(format!("Hey there, {name:?}!")).await;
}
