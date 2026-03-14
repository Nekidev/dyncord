use std::env;

use dyncord::Bot;
use dyncord::commands::Command;
use dyncord::commands::prefixed::arguments::{IntoArgument, ParsingError};
use dyncord::commands::prefixed::context::PrefixedContext;
use dyncord::utils::DynFuture;
use twilight_gateway::Intents;

#[tokio::main]
async fn main() {
    let bot = Bot::new(())
        .with_prefix(".")
        .intents(Intents::GUILD_MESSAGES)
        .intents(Intents::MESSAGE_CONTENT)
        .command(Command::prefixed("hello", hello));

    bot.run(env::var("TOKEN").unwrap()).await.unwrap();
}

async fn hello(ctx: PrefixedContext, name: Name) {
    ctx.send(format!("Hello, {}!", name.pretty()))
        .await
        .unwrap();
}

struct Name(String, String);

impl Name {
    fn pretty(&self) -> String {
        let first = self
            .0
            .chars()
            .next()
            .unwrap_or_default()
            .to_uppercase()
            .to_string();

        format!("{}. {}", first, self.1)
    }
}

impl IntoArgument<()> for Name {
    fn into_argument(
        _ctx: PrefixedContext<()>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let mut parts = args.splitn(3, ' ').collect::<Vec<&str>>();

            if parts.len() < 2 {
                return Err(ParsingError::InvalidArgument);
            }

            if parts.len() == 2 {
                parts.push("");
            }

            Ok((
                Name(parts[0].to_string(), parts[1].to_string()),
                parts[2].to_string(),
            ))
        })
    }
}
