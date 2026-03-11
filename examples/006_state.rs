use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use dyncord::Bot;
use dyncord::commands::Command;
use dyncord::commands::context::CommandContext;
use dyncord::events::{EventContext, MessageCreate};
use twilight_gateway::Intents;

#[tokio::main]
async fn main() {
    let bot = Bot::new(CounterState::default())
        .with_prefix("!")
        .intents(Intents::GUILD_MESSAGES)
        .intents(Intents::MESSAGE_CONTENT)
        .command(Command::build("count", count_command))
        .command(Command::build("reset", reset_command))
        .on_event(on_message);

    bot.run(env::var("TOKEN").unwrap()).await;
}

async fn on_message(ctx: EventContext<CounterState, MessageCreate>) {
    ctx.state.counter.fetch_add(1, Ordering::SeqCst);
}

async fn count_command(ctx: CommandContext<CounterState>) {
    let count = ctx.state.counter.load(Ordering::SeqCst);

    ctx.reply(format!("Message count: {}", count))
        .await
        .unwrap();
}

async fn reset_command(ctx: CommandContext<CounterState>) {
    ctx.state.counter.store(0, Ordering::SeqCst);

    ctx.reply("Message count has been reset!").await.unwrap();
}

#[derive(Default, Clone)]
struct CounterState {
    counter: Arc<AtomicUsize>,
}
