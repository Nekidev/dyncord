use std::env;

use dyncord::Bot;
use dyncord::builtin::cache::inmemory::InMemoryCache;
use dyncord::commands::Command;
use dyncord::commands::prefixed::context::PrefixedContext;
use twilight_gateway::Intents;

#[tokio::main]
async fn main() {
    let bot = Bot::new(())
        .with_prefix(".")
        .intents(Intents::GUILD_MESSAGES)
        .intents(Intents::MESSAGE_CONTENT)
        .command(Command::prefixed("hello", hello))
        .with_cache(InMemoryCache::default());

    bot.run(env::var("TOKEN").unwrap()).await.unwrap();
}

async fn hello(ctx: PrefixedContext, name: String, age: u32) {
    ctx.send(format!("Hello, {name} who's {age} years old!"))
        .await
        .unwrap();
}
