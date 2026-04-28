use std::env;

use dyncord::Bot;
use dyncord::commands::Command;
use dyncord::commands::slash::arguments::Argument;
use dyncord::commands::slash::context::SlashContext;

#[tokio::main]
async fn main() {
    let bot = Bot::new(()).with_prefix(".").command(
        Command::slash("hello", hello)
            .description("Says hi back.")
            .argument(
                Argument::string("name").description("Your name, to sell it to *ahem* to say hi."),
            )
            .argument(Argument::integer("age").description("Your age.").optional()),
    );

    bot.run(env::var("TOKEN").unwrap()).await.unwrap();
}

async fn hello(ctx: SlashContext, name: String, age: Option<u8>) {
    ctx.respond(format!("Hey there, {name} aged {}!", age.unwrap_or(255)))
        .await
        .unwrap();
}
