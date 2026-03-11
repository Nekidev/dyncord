# Dyncord

Dyncord is a Rust library for building Discord bots with an axum-like API.

Twilight is a Rust library for interacting with the Discord API. It however does not provide a
high-level API and utilities to build bots, so you have to build your own framework on top of
it. Dyncord is a library that provides those high-level APIs and utilities to build bots,
hiding the low-level details of the Discord API to provide a more pleasant DX for Discord bot
developers.

Why "dyncord"? Because I expected it to use a lot of `dyn` internally. Ironically, it ended up
using very little `dyn`.

# Quick Overview

A minimal bot using Dyncord looks like this:

```rs
use dyncord::{Bot, Intents};
use dyncord::commands::Command;
use dyncord::commands::context::CommandContext;

#[tokio::main]
async fn main() {
    let bot = Bot::new(())
        .intents(Intents::GUILD_MESSAGES | INTENTS::MESSAGE_CONTENT)
        .with_prefix(">")
        .command(Command::new("hello", hello));

    bot.run("your-token").await.unwrap();
}

async fn hello(ctx: CommandContext) {
    ctx.send("Hello, world!").await.unwrap();
}
```

Then on Discord, just send `>hello` in a channel the bot has access to and it will reply with
`Hello, world!`.

Taking arguments is simple. Just add them to the handler function:

```rs
async fn hello(ctx: CommandContext, name: String) {
    ctx.send(format!("Hello, {name}!")).await.unwrap();
}
```

# Installation

Installing is simple. Just add `dyncord` to your `Cargo.toml`:

```sh
cargo add dyncord
```

You'll also need a runtime to run the bot. We only support `tokio` for now, so add it to your
`Cargo.toml` as well:

```sh
cargo add tokio -F full
```

# Quick Start

To start with, create a [`Bot`] instance with [`Bot::new()`](Bot::new). The only argument it
takes is the bot's state, which can be any type you want (`Send + Sync + Clone`). For this
example, we'll just use `()`. We'll also use `.` as the bot's prefix.

```rs
#[tokio::main]
async fn main() {
    let bot = Bot::new(()).with_prefix(".");
}
```

Great! Now we have a bot instance, let's just add our bot's token and get it to run.

```rs
bot.run("token").await.unwrap();
```

Check Discord and you'll see your bot has come online. Well done! Now, let's add a command to
our bot.

Command handlers are simple to define. The simplest form of a command handler is an async
function that takes a [`Context`](commands::context::CommandContext) as its only argument. For
example:

```rs
async fn ping(ctx: Context) {
    ctx.send("pong").await.unwrap();
}
```

To add that command to our bot, we just need to create a
[`Command`](commands::Command) instance and pass it to the bot's [`command`](Bot::command)
method:

```
let bot = Bot::new(()).with_prefix(".").command(Command::new("ping", ping));
```

`"ping"` is the command's name, used to invoke the command. So in this case, sending `.ping` in
a channel the bot has access to will trigger the command and make the bot reply with `pong`.

For message commands to run properly, the bot needs to have the `MESSAGE_CONTENT` intent
enabled and any intents required to receive messages. For example, `GUILD_MESSAGES` for
messages sent in servers.

Let's add those to our bot:

```rs
let bot = Bot::new(())
    .intents(Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT)
    .with_prefix(".")
    .command(Command::new("ping", ping));
```

Now, when you send `.ping` in a channel the bot has access to, it will reply with `pong`. Good!

Last for this quick start, let's see how to take arguments. Just add them to the handler
function as normal arguments and they'll be parsed and passed to the handler when the command
is invoked.

For example, if we want to take a user as an argument, we can just add a `User` argument to the
handler function:

```rs
async fn hello(ctx: Context, name: String) {
    ctx.send(format!("Hello, {name}!")).await.unwrap();
}
```

When invoking the command, just mention a user after the command name and the bot will parse
the mentioned user and pass it to the handler. Adding the handler above to the bot and invoking
it with `.hello @someuser` will make the bot reply with `Hello, @someuser!`.

# WIP

Dyncord is a work in progress, extremely early in development, and certainly not ready for
production use. Any help is appreciated, whether it's testing, bug reports, or contributions.
