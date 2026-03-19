use std::env;

use dyncord::builtin::cache::redis::RedisCache;
use dyncord::commands::Command;
use dyncord::commands::prefixed::context::PrefixedContext;
use dyncord::wrappers::types::users::User;
use dyncord::{Bot, Intents};
use redis::Client;
use redis::aio::ConnectionManager;

#[tokio::main]
async fn main() {
    let client = Client::open("redis://localhost/0").unwrap();
    let connection = ConnectionManager::new(client).await.unwrap();

    let bot = Bot::new(())
        .with_prefix(".")
        .intents(Intents::GUILD_MESSAGES)
        .intents(Intents::MESSAGE_CONTENT)
        .command(Command::prefixed("hello", hello))
        .with_cache(RedisCache::new(connection));

    bot.run(env::var("TOKEN").unwrap()).await.unwrap();
}

async fn hello(ctx: PrefixedContext, user: User) {
    ctx.send(format!("Hello, {}!", user.name_display()))
        .await
        .unwrap();
}
