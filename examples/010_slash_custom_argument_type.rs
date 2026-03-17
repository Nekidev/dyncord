use std::env;

use dyncord::Bot;
use dyncord::commands::Command;
use dyncord::commands::errors::ArgumentError;
use dyncord::commands::slash::arguments::{Argument, ArgumentType, IntoArgument};
use dyncord::commands::slash::context::SlashContext;
use dyncord::utils::{DynFuture, pinbox};
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue,
};

#[tokio::main]
async fn main() {
    let bot = Bot::new(()).with_prefix(".").command(
        Command::slash("hello", hello)
            .description("Says hi back.")
            .argument(
                Argument::string("name").description("Your name, to sell it to *ahem* to say hi."),
            ),
    );

    bot.run(env::var("TOKEN").unwrap()).await.unwrap();
}

async fn hello(ctx: SlashContext, name: Name) {
    ctx.respond(format!("Hey there, {} {}!", name.0, name.1))
        .await
        .unwrap();
}

struct Name(String, String);

impl IntoArgument<()> for Name {
    fn into_argument_primitive(
        _ctx: SlashContext<()>,
        argument: Option<CommandDataOption>,
    ) -> DynFuture<'static, Result<Self, ArgumentError>> {
        if let Some(argument) = argument {
            if let CommandOptionValue::String(argument) = argument.value {
                match argument.split_once(' ') {
                    Some((first, last)) => pinbox(Ok(Name(first.into(), last.into()))),
                    None => pinbox(Err(ArgumentError::Misformatted)),
                }
            } else {
                pinbox(Err(ArgumentError::Misformatted))
            }
        } else {
            pinbox(Err(ArgumentError::Missing))
        }
    }

    fn r#type() -> (ArgumentType, bool) {
        (ArgumentType::String, false)
    }
}
