//! Slash command argument builders and parsers.

use std::collections::HashMap;

use thiserror::Error;
use twilight_model::application::command::{CommandOption, CommandOptionType};
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue,
};

use crate::commands::slash::context::SlashContext;
use crate::state::StateBound;
use crate::utils::{DynFuture, pinbox};

/// A unified API to build slash-command argument metadata.
///
/// It has multiple associated functions, one per type of argument that can be built.
pub struct Argument;

impl Argument {
    /// Initializes a string argument builder.
    ///
    /// Arguments:
    /// * `name` - The argument's name, between 1 and 32 characters long.
    ///
    /// Returns:
    /// [`StringArgumentBuilder`] - The new string argument builder.
    pub fn string(name: impl Into<String>) -> StringArgumentBuilder {
        StringArgumentBuilder::new(name)
    }
}

/// Slash-command argument metadata.
#[derive(Clone)]
pub enum ArgumentMeta {
    String(StringArgument),
    OptionalString(OptionalStringArgument),
}

impl ArgumentMeta {
    /// Returns the inner value's argument name.
    /// 
    /// Returns:
    /// [`&String`] -> The inner value's argument name.
    pub fn name(&self) -> &String {
        match self {
            Self::String(inner) => &inner.name,
            Self::OptionalString(inner) => &inner.name,
        }
    }

    /// Returns the argument type of the current argument.
    /// 
    /// Returns:
    /// [`ArgumentType`] - The current argument's type.
    pub fn r#type(&self) -> ArgumentType {
        match self {
            Self::String(_) => ArgumentType::String,
            Self::OptionalString(_) => ArgumentType::OptionalString,
        }
    }
}

impl From<ArgumentMeta> for CommandOption {
    fn from(value: ArgumentMeta) -> Self {
        match value {
            ArgumentMeta::String(inner) => inner.into(),
            ArgumentMeta::OptionalString(inner) => inner.into(),
        }
    }
}

/// Slash-command argument types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentType {
    String,
    OptionalString,
}

#[derive(Clone)]
pub struct StringArgument {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    min_length: Option<u16>,
    max_length: Option<u16>,
}

impl From<StringArgument> for CommandOption {
    fn from(value: StringArgument) -> Self {
        CommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: value.description,
            description_localizations: Some(value.description_i18n),
            kind: CommandOptionType::String,
            min_length: value.min_length,
            max_length: value.max_length,
            min_value: None,
            max_value: None,
            name: value.name,
            name_localizations: Some(value.name_i18n),
            options: None,
            required: Some(true),
        }
    }
}

pub struct StringArgumentBuilder {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    min_length: Option<u16>,
    max_length: Option<u16>,
}

impl StringArgumentBuilder {
    fn new(name: impl Into<String>) -> Self {
        StringArgumentBuilder {
            name: name.into(),
            name_i18n: HashMap::new(),
            description: String::from("A Dyncord argument."),
            description_i18n: HashMap::new(),
            min_length: None,
            max_length: None,
        }
    }

    pub fn name_i18n(mut self, lang: impl Into<String>, name: impl Into<String>) -> Self {
        self.name_i18n.insert(lang.into(), name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn description_i18n(
        mut self,
        lang: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.description_i18n
            .insert(lang.into(), description.into());
        self
    }

    pub fn min_length(mut self, length: u16) -> Self {
        self.min_length = Some(length);
        self
    }

    pub fn max_length(mut self, length: u16) -> Self {
        self.max_length = Some(length);
        self
    }

    pub fn optional(self) -> OptionalStringArgumentBuilder {
        OptionalStringArgumentBuilder {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            min_length: self.min_length,
            max_length: self.max_length,
        }
    }

    fn build(self) -> StringArgument {
        StringArgument {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            min_length: self.min_length,
            max_length: self.max_length,
        }
    }
}

impl From<StringArgumentBuilder> for StringArgument {
    fn from(value: StringArgumentBuilder) -> Self {
        value.build()
    }
}

impl From<StringArgument> for ArgumentMeta {
    fn from(value: StringArgument) -> Self {
        ArgumentMeta::String(value)
    }
}

impl From<StringArgumentBuilder> for ArgumentMeta {
    fn from(value: StringArgumentBuilder) -> Self {
        ArgumentMeta::String(value.build())
    }
}

#[derive(Clone)]
pub struct OptionalStringArgument {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    min_length: Option<u16>,
    max_length: Option<u16>,
}

impl From<OptionalStringArgument> for CommandOption {
    fn from(value: OptionalStringArgument) -> Self {
        CommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: value.description,
            description_localizations: Some(value.description_i18n),
            kind: CommandOptionType::String,
            min_length: value.min_length,
            max_length: value.max_length,
            min_value: None,
            max_value: None,
            name: value.name,
            name_localizations: Some(value.name_i18n),
            options: None,
            required: Some(false),
        }
    }
}

pub struct OptionalStringArgumentBuilder {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    min_length: Option<u16>,
    max_length: Option<u16>,
}

impl OptionalStringArgumentBuilder {
    pub fn name_i18n(mut self, lang: impl Into<String>, name: impl Into<String>) -> Self {
        self.name_i18n.insert(lang.into(), name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn description_i18n(
        mut self,
        lang: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.description_i18n
            .insert(lang.into(), description.into());
        self
    }

    pub fn min_length(mut self, length: u16) -> Self {
        self.min_length = Some(length);
        self
    }

    pub fn max_length(mut self, length: u16) -> Self {
        self.max_length = Some(length);
        self
    }

    fn build(self) -> OptionalStringArgument {
        OptionalStringArgument {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            min_length: self.min_length,
            max_length: self.max_length,
        }
    }
}

impl From<OptionalStringArgumentBuilder> for OptionalStringArgument {
    fn from(value: OptionalStringArgumentBuilder) -> Self {
        value.build()
    }
}

impl From<OptionalStringArgument> for ArgumentMeta {
    fn from(value: OptionalStringArgument) -> Self {
        ArgumentMeta::OptionalString(value)
    }
}

impl From<OptionalStringArgumentBuilder> for ArgumentMeta {
    fn from(value: OptionalStringArgumentBuilder) -> Self {
        ArgumentMeta::OptionalString(value.build())
    }
}

#[derive(Debug, Error)]
pub enum ArgumentError {
    #[error("An argument was received, but its value was not of the expected type.")]
    IncorrectType,

    #[error("A required argument was not received together with the Discord event.")]
    Missing,

    #[error(
        "An argument was internally queried since we got a slash command invocation, but the argument's metadata was missing."
    )]
    MissingMeta,

    #[error("An argument had the correct Discord-native type, but the value passed was invalid.")]
    InvalidValue,
}

pub trait IntoArgument<State>: Sized + Send + Sync
where
    State: StateBound,
{
    /// Converts a raw twilight [`CommandDataOption`] into the type taken by slash command handlers
    /// as arguments.
    ///
    /// Arguments:
    /// * `ctx` - The slash command context of the current command execution.
    /// * `argument` - The argument being parsed, or [`None`] if the argument was declared but not
    ///   received.
    ///
    /// Returns:
    /// [`Result<Self, ArgumentError>`] - The parsed primitive, or an error if it failed to be
    /// parsed.
    fn into_argument_primitive(
        ctx: SlashContext<State>,
        argument: Option<CommandDataOption>,
    ) -> DynFuture<'static, Result<Self, ArgumentError>>;

    /// The type of the argument from which this type is parsed.
    ///
    /// This is used to make sure commands have been configured correctly when starting the bot.
    ///
    /// Returns:
    /// [`ArgumentType`] - The Discord-native type of the argument being parsed.
    fn r#type() -> ArgumentType;
}

impl<State> IntoArgument<State> for String
where
    State: StateBound,
{
    fn into_argument_primitive(
        _ctx: SlashContext<State>,
        argument: Option<CommandDataOption>,
    ) -> DynFuture<'static, Result<Self, ArgumentError>> {
        if let Some(argument) = argument {
            if let CommandOptionValue::String(value) = argument.value {
                pinbox(Ok(value))
            } else {
                pinbox(Err(ArgumentError::IncorrectType))
            }
        } else {
            pinbox(Err(ArgumentError::Missing))
        }
    }

    fn r#type() -> ArgumentType {
        ArgumentType::String
    }
}

impl<State> IntoArgument<State> for Option<String>
where
    State: StateBound,
{
    fn into_argument_primitive(
        _ctx: SlashContext<State>,
        argument: Option<CommandDataOption>,
    ) -> DynFuture<'static, Result<Self, ArgumentError>> {
        if let Some(argument) = argument {
            if let CommandOptionValue::String(value) = argument.value {
                pinbox(Ok(Some(value)))
            } else {
                pinbox(Err(ArgumentError::IncorrectType))
            }
        } else {
            pinbox(Ok(None))
        }
    }

    fn r#type() -> ArgumentType {
        ArgumentType::OptionalString
    }
}
