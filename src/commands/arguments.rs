use std::fmt::Display;

use thiserror::Error;

use crate::DynFuture;
use crate::commands::context::CommandContext;
use crate::state::StateBound;

/// An error occurred while parsing an argument for a command.
#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("The argument provided is invalid.")]
    InvalidArgument,

    #[error("Not enough arguments were provided.")]
    MissingArgument,
}

/// Implements conversion from a raw message into a command's argument.
pub trait IntoArgument<State = ()>: Sized + Send + Sync
where
    State: StateBound,
{
    /// Converts a raw message into a command's argument.
    ///
    /// This function takes two arguments, the command context and the raw arguments. It returns
    /// the parsed argument and the remaining raw arguments if successful, or a [`ParsingError`]
    /// if parsing the argument failed.
    ///
    /// For example, to parse a `String` argument (which takes one word), the implementation looks
    /// like this:
    ///
    /// ```
    /// fn into_argument(
    ///     _ctx: CommandContext<State>,
    ///     args: String,
    /// ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
    ///     Box::pin(async move {
    ///         let trimmed = args.trim_start();
    ///
    ///         match trimmed.split_once(' ') {
    ///             Some((arg, remaining)) => Ok((arg.to_string(), remaining.to_string())),
    ///             None => {
    ///                 if args.is_empty() {
    ///                     Err(ParsingError::MissingArgument)
    ///                 } else {
    ///                     Ok((args.to_string(), "".to_string()))
    ///                 }
    ///             }
    ///         }
    ///     })
    /// }
    /// ```
    ///
    /// Arguments:
    /// * `ctx` - The command context, which contains information about the message, channel,
    ///   guild, etc.
    /// * `args` - The raw arguments passed to the command, which can be parsed into the command's
    ///   arguments.
    ///
    /// Returns:
    /// * `Ok((argument, remaining_args))` - The parsed argument and the remaining raw arguments if
    ///   parsing was successful.
    /// * `Err(ParsingError)` - A parsing error if parsing the argument failed.
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>>;
}

impl<State> IntoArgument<State> for String
where
    State: StateBound,
{
    fn into_argument(
        _ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let trimmed = args.trim_start();

            if let Some(argument) = parse_token(trimmed) {
                let remaining = trimmed[(argument.value().last + 1)..].to_string();

                Ok((argument.to_string(), remaining))
            } else {
                Err(ParsingError::MissingArgument)
            }
        })
    }
}

#[derive(Debug)]
enum Token {
    String(TokenValue),
    InSingleQuote(TokenValue),
    InDoubleQuote(TokenValue),
    Spaces(TokenValue),
}

impl Token {
    /// Returns the inner [`TokenValue`] of this token.
    ///
    /// Returns:
    /// [`TokenValue`] - The token's value and metadata.
    fn value(&self) -> &TokenValue {
        match self {
            Self::InDoubleQuote(value) => value,
            Self::InSingleQuote(value) => value,
            Self::Spaces(value) => value,
            Self::String(value) => value,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(inner) => f.write_str(&inner.value),
            Self::Spaces(inner) => f.write_str(&inner.value),
            Self::InDoubleQuote(inner) => f.write_str(&inner.value),
            Self::InSingleQuote(inner) => f.write_str(&inner.value),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct TokenValue {
    value: String,
    first: usize,
    last: usize,
}

/// Parses the first token of raw arguments.
///
/// This tokenizer parses into 4 different token types:
/// - [`Token::String`] - A string word, unquoted.
/// - [`Token::InSingleQuote`] - A single-quote-quoted string. E.g. `'hello world!'`.
/// - [`Token::InDoubleQuote`] - A double-quote-quoted string. E.g. `"hello world!"`.
/// - [`Token::Spaces`] - One or more spaces, separating string and quoted-string tokens.
///
/// This function is an extract of a tokenizer that parses all tokens in a string. This one just
/// parses and returns the first found token.
///
/// It delegates the parsing of each token type to each of the `parse_*` functions (defined below),
/// [`parse_string`], [`parse_space`], [`parse_single_quote`], and [`parse_double_quote`]. Each of
/// them takes a mutable reference to the cursor (`i`, in this function `&mut 0` since it only
/// parses one token) and advances it as it consumes characters from `args` into the token being
/// parsed.
///
/// This tokenizer is used to parse quoted strings as argument values in commands. For example,
/// `!hello "Mike Wazowski"` makes `Mike Wazowski` the first [`String`] argument of this command.
/// It also supports escapes and single-quote quoting, meaning `!hello 'Mike Wazowski'` also works
/// like the first example, and `!echo 'it\'s tuesday!'`'s first argument will be properly parsed
/// as `it's tuesday!`.
///
/// Some examples of how the tokenizer will convert arguments into tokens are:
///
/// - `hello "world"` -> `[String("hello"), Spaces(" "), InDoubleQuotes("world")]`
/// - `hello"world"` -> `[String("hello\"world\"")]`
/// - `hello \"world\"` -> `[String("hello"), Spaces(" "), String("\"world\"")]`
///
/// Note that this tokenizer function only parses the first token, so only the first token of those
/// arrays is returned in practice.
///
/// Arguments:
/// * `args` - The raw args from which to parse the first token.
///
/// Returns:
/// [`Option<Token>`] - The first token parsed, if `args` wasn't empty.
fn parse_token(args: &str) -> Option<Token> {
    if args.is_empty() {
        return None;
    }

    let chars: Vec<_> = args.chars().collect();

    match chars[0] {
        ' ' => Some(parse_space(&chars, &mut 0)),
        '\'' => Some(parse_single_quote(&chars, &mut 0)),
        '"' => Some(parse_double_quote(&chars, &mut 0)),
        _ => Some(parse_string(&chars, &mut 0)),
    }
}

/// Parses a raw string into a [`Token::String`].
///
/// [`Token::String`] tokens are unquoted strings. They support escaping quotes, e.g.
/// `\"hello\"` -> `Token::String("\"hello\"")`, but quotes inside this token are treated like
/// literals. This token ends when either there's no characters left to parse or a white space is
/// found.
///
/// This function will advance the cursor as chars are being parsed into the token, and the cursor
/// will be `last_token_index + 1` when the function returns.
///
/// Note: The cursor MUST be the starting index of the string token to parse when this function is
///       called. Not guaranteeing so before calling this function will cause the wrong characters,
///       potentially not belonging in a [`Token::String`], to be parsed into a [`Token::String`].
///
/// Arguments:
/// * `chars` - A slice pointing to all chars being parsed.
/// * `i` - A mutable reference to the parsing cursor.
///
/// Returns:
/// [`Token::String`] - The string token parsed.
fn parse_string(chars: &[char], i: &mut usize) -> Token {
    let mut current = String::new();

    let first_i = *i;

    while *i < chars.len() {
        let current_char = chars[*i];

        match current_char {
            ' ' => {
                break;
            }
            '\\' => {
                *i += 1;

                if ['\'', '"', '\\'].contains(&chars[*i]) {
                    current.push(chars[*i]);
                } else {
                    current.push('\\');
                    current.push(chars[*i]);
                }
            }
            ch => {
                current.push(ch);
            }
        }

        *i += 1;
    }

    Token::String(TokenValue {
        value: current,
        first: first_i,
        last: *i - 1,
    })
}

/// Parses zero or more white spaces into a [`Token::Spaces`].
///
/// Even though the [`parse_token`] function indicates that [`Token::Spaces`] tokens contain one or
/// more white spaces, this function may return a [`Token::Spaces`] containing an empty string if
/// the passed cursor does not point to a white space when this function is called.
///
/// This function will advance the cursor as chars are being parsed into the token, and the cursor
/// will be `last_token_index + 1` when the function returns.
///
/// Arguments:
/// * `chars` - A slice pointing to all chars being parsed.
/// * `i` - A mutable reference to the parsing cursor.
///
/// Returns:
/// [`Token::Spaces`] - The spaces token parsed.
fn parse_space(chars: &[char], i: &mut usize) -> Token {
    let mut current = String::new();

    let first_i = *i;

    while *i < chars.len() && chars[*i] == ' ' {
        current.push(chars[*i]);
        *i += 1;
    }

    Token::Spaces(TokenValue {
        value: current,
        first: first_i,
        last: *i - 1,
    })
}

/// Parses a single-quote-quoted token, or a [`Token::String`] if there's no closing quote.
///
/// [`Token::InSingleQuote`] tokens represent single-quote-quoted strings. For example,
/// `'hello world'`. It supports escaping such quotes, and escaping backslashes not to escape
/// single quotes.
/// 
/// If the single-quote-quoted string being parsed ends up not having a closing quote, this'll
/// fall back to parsing the token as a [`Token::String`] using [`parse_string`].
/// 
/// This function will advance the cursor as chars are being parsed into the token, and the cursor
/// will be `last_token_index + 1` when the function returns.
///
/// Note: The cursor MUST be the starting index of the single-quote-quoted token to parse when this
///       function is called. This means `i` should be the index of a `'\''` char in `chars`. Not
///       guaranteeing so before calling this function to panic.
/// 
/// Arguments:
/// * `chars` - A slice pointing to all chars being parsed.
/// * `i` - A mutable reference to the parsing cursor.
/// 
/// Returns:
/// * [`Token::InSingleQuote`] - If the token was successfully parsed as a single-quote-quoted
///   string.
/// * [`Token::String`] - If the token didn't have a closing single quote. E.g. `'hello`.
/// 
/// Panics:
/// * If `i` is not the index of a `'\''` char in `chars` when the function is called.
fn parse_single_quote(chars: &[char], i: &mut usize) -> Token {
    // In case we don't find a closing quote, this will let us restart as a string.
    let first_i = *i;

    let mut current = String::new();

    if chars[*i] != '\'' {
        unreachable!("The first character of a single-quote string must be a single quote (').");
    }

    // Skip the leading single quote.
    *i += 1;

    while *i < chars.len() {
        match chars[*i] {
            '\\' => {
                *i += 1; // Let's check what the following character is.

                if let Some(next) = chars.get(*i) {
                    if ['\\', '\''].contains(next) {
                        // The next char is escape-able, we push it directly.
                        current.push(*next);
                    } else {
                        // The next char is not escape-able, so the backslash is just a backslash.
                        current.push('\\');
                        current.push(*next);
                    }
                } else {
                    break; // There's no next loop run, we stop before *i += 1 after this `match`.
                }
            }
            '\'' => {
                // Point to the next char for the next parser and return what we found.
                *i += 1;
                return Token::InSingleQuote(TokenValue {
                    value: current,
                    first: first_i,
                    last: *i - 1,
                });
            }
            ch => {
                current.push(ch);
            }
        }

        *i += 1;
    }

    // In single quote, but we never found the closing quote. This was a string all the time.
    *i = first_i;
    parse_string(chars, i)
}

/// Parses a double-quote-quoted token, or a [`Token::String`] if there's no closing quote.
///
/// [`Token::InDoubleQuote`] tokens represent double-quote-quoted strings. For example,
/// `"hello world"`. It supports escaping such quotes, and escaping backslashes not to escape
/// double quotes.
/// 
/// If the double-quote-quoted string being parsed ends up not having a closing quote, this'll
/// fall back to parsing the token as a [`Token::String`] using [`parse_string`].
/// 
/// This function will advance the cursor as chars are being parsed into the token, and the cursor
/// will be `last_token_index + 1` when the function returns.
///
/// Note: The cursor MUST be the starting index of the double-quote-quoted token to parse when this
///       function is called. This means `i` should be the index of a `'"'` char in `chars`. Not
///       guaranteeing so before calling this function to panic.
/// 
/// Arguments:
/// * `chars` - A slice pointing to all chars being parsed.
/// * `i` - A mutable reference to the parsing cursor.
/// 
/// Returns:
/// * [`Token::InDoubleQuote`] - If the token was successfully parsed as a double-quote-quoted
///   string.
/// * [`Token::String`] - If the token didn't have a closing single quote. E.g. `"hello`.
/// 
/// Panics:
/// * If `i` is not the index of a `'"'` char in `chars` when the function is called.
fn parse_double_quote(chars: &[char], i: &mut usize) -> Token {
    // In case we don't find a closing quote, this will let us restart as a string.
    let first_i = *i;

    let mut current = String::new();

    if chars[*i] != '"' {
        unreachable!("The first character of a double-quote string must be a double quote (\").");
    }

    // Skip the leading double quote.
    *i += 1;

    while *i < chars.len() {
        match chars[*i] {
            '\\' => {
                *i += 1; // Let's check what the following character is.

                if let Some(next) = chars.get(*i) {
                    if ['\\', '"'].contains(next) {
                        // The next char is escape-able, we push it directly.
                        current.push(*next);
                    } else {
                        // The next char is not escape-able, so the backslash is just a backslash.
                        current.push('\\');
                        current.push(*next);
                    }
                } else {
                    break; // There's no next loop run, we stop before *i += 1 after this `match`.
                }
            }
            '"' => {
                // Point to the next char for the next parser and return what we found.
                *i += 1;
                return Token::InDoubleQuote(TokenValue {
                    value: current,
                    first: first_i,
                    last: *i - 1,
                });
            }
            ch => {
                current.push(ch);
            }
        }

        *i += 1;
    }

    // In double quote, but we never found the closing quote. This was a string all the time.
    *i = first_i;
    parse_string(chars, i)
}

/// Takes all remaining raw arguments as a single string argument.
///
/// For example, if a command is invoked with `.echo Hello, world!`, the `GreedyString` argument
/// will be parsed as `Hello, world!` instead of just `Hello,`.
///
/// To use it in a handler, just add it as an argument like follows:
///
/// ```
/// async fn echo(ctx: CommandContext, GreedyString(message): GreedyString) {
///     ctx.send(message).await.unwrap();
/// }
/// ```
pub struct GreedyString(pub String);

impl<State> IntoArgument<State> for GreedyString
where
    State: StateBound,
{
    fn into_argument(
        _ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move { Ok((GreedyString(args.trim_start().to_string()), "".to_string())) })
    }
}

impl<State> IntoArgument<State> for char
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            let mut chars = arg.chars();

            match (chars.next(), chars.next()) {
                (Some(c), None) => Ok((c, remaining)),
                _ => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for i8
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for i16
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for i32
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for i64
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for i128
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for isize
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for u8
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for u16
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for u32
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for u64
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for u128
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for usize
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for f32
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for f64
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.parse::<Self>() {
                Ok(num) => Ok((num, remaining)),
                Err(_) => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State> IntoArgument<State> for bool
where
    State: StateBound,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            let (arg, remaining) = String::into_argument(ctx, args).await?;
            match arg.to_lowercase().as_str() {
                "true" | "y" | "yes" | "1" | "on" => Ok((true, remaining)),
                "false" | "n" | "no" | "0" | "off" => Ok((false, remaining)),
                _ => Err(ParsingError::InvalidArgument),
            }
        })
    }
}

impl<State, T> IntoArgument<State> for Option<T>
where
    State: StateBound,
    T: IntoArgument<State>,
{
    fn into_argument(
        ctx: CommandContext<State>,
        args: String,
    ) -> DynFuture<'static, Result<(Self, String), ParsingError>> {
        Box::pin(async move {
            match T::into_argument(ctx, args.clone()).await {
                Ok((arg, remaining)) => Ok((Some(arg), remaining)),
                Err(_) => Ok((None, args)),
            }
        })
    }
}
