//! Slash command argument builders and parsers.
//!
//! Argument metadata builders are initialized via the [`Argument`] interface. The
//! currently-supported argument types are:
//!
//! - [`Argument::string`] - [`String`]
//! - [`Argument::integer`] - [`i8`], [`i16`], [`i32`], [`i64`], [`isize`], [`u8`], [`u16`],
//!   [`u32`], [`u64`], [`usize`]
//! - [`Argument::float`] - [`f32`], [`f64`]
//! - [`Argument::boolean`] - [`bool`]
//! - [`Argument::user`] - [`User`]
//! - [`Argument::channel`] - [`Channel`]
//! - [`Argument::role`] - [`Role`]
//!
//! Each of those functions returns a builder specific to the argument type. They differ in some
//! properties, but they all share the following functions:
//!
//! - `::name_i18n()`
//! - `::description()`
//! - `::description_i18n()`
//! - `::optional()`
//!
//! I18n stands for internationalization, i.e. translation. Call `*_i18n()` methods once per
//! language, for example:
//!
//! ```
//! Argument::string("name")
//!     .name_i18n("es-ES", "nombre")
//!     .name_i18n("ja-JP", "名前")
//!     .name_i18n("fr-FR", "nom");
//! ```
//!
//! `::optional()` changes the type of argument your handler will receive from `T` to `Option<T>`.
//! For example,
//!
//! ```
//! async fn handle_string_required(_ctx: SlashContext, _name: String) {}
//! async fn handle_string_optional(_ctx: SlashContext, _name: Option<String>) {}
//!
//! Command::slash("required", handle_string_required)
//!     .argument(Argument::string("name"));
//!
//! Command::slash("optional", handle_string_optional)
//!     .argument(Argument::string("name").optional());
//! ```

use std::collections::HashMap;

use num::FromPrimitive;
use twilight_model::application::command::{
    CommandOption as InnerCommandOption, CommandOptionType as InnerCommandOptionType,
    CommandOptionValue as InnerCommandOptionValue,
};
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue,
};

use crate::commands::errors::ArgumentError;
use crate::commands::slash::context::SlashContext;
use crate::state::StateBound;
use crate::utils::{DynFuture, pinbox};
use crate::wrappers::types::channels::Channel;
use crate::wrappers::types::roles::Role;
use crate::wrappers::types::users::User;

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

    /// Initializes an integer argument builder.
    ///
    /// Arguments:
    /// * `name` - The argument's name, between 1 and 32 characters long.
    ///
    /// Returns:
    /// [`IntegerArgumentBuilder`] - The new integer argument builder.
    pub fn integer(name: impl Into<String>) -> IntegerArgumentBuilder {
        IntegerArgumentBuilder::new(name)
    }

    /// Initializes a float argument builder.
    ///
    /// Arguments:
    /// * `name` - The argument's name, between 1 and 32 characters long.
    ///
    /// Returns:
    /// [`FloatArgumentBuilder`] - The new float argument builder.
    pub fn float(name: impl Into<String>) -> FloatArgumentBuilder {
        FloatArgumentBuilder::new(name)
    }

    /// Initializes a boolean argument builder.
    ///
    /// Arguments:
    /// * `name` - The argument's name, between 1 and 32 characters long.
    ///
    /// Returns:
    /// [`BooleanArgumentBuilder`] - The new boolean argument builder.
    pub fn boolean(name: impl Into<String>) -> BooleanArgumentBuilder {
        BooleanArgumentBuilder::new(name)
    }

    /// Initializes a user argument builder.
    ///
    /// Arguments:
    /// * `name` - The argument's name, between 1 and 32 characters long.
    ///
    /// Returns:
    /// [`UserArgumentBuilder`] - The new user argument builder.
    pub fn user(name: impl Into<String>) -> UserArgumentBuilder {
        UserArgumentBuilder::new(name)
    }

    /// Initializes a channel argument builder.
    ///
    /// Arguments:
    /// * `name` - The argument's name, between 1 and 32 characters long.
    ///
    /// Returns:
    /// [`ChannelArgumentBuilder`] - The new channel argument builder.
    pub fn channel(name: impl Into<String>) -> ChannelArgumentBuilder {
        ChannelArgumentBuilder::new(name)
    }

    /// Initializes a role argument builder.
    ///
    /// Arguments:
    /// * `name` - The argument's name, between 1 and 32 characters long.
    ///
    /// Returns:
    /// [`RoleArgumentBuilder`] - The new role argument builder.
    pub fn role(name: impl Into<String>) -> RoleArgumentBuilder {
        RoleArgumentBuilder::new(name)
    }
}

/// Slash-command argument metadata.
#[derive(Clone)]
pub enum ArgumentMeta {
    String(StringArgument),
    Float(FloatArgument),
    Integer(IntegerArgument),
    Boolean(BooleanArgument),
    User(UserArgument),
    Channel(ChannelArgument),
    Role(RoleArgument),
}

impl ArgumentMeta {
    /// Returns the inner value's argument name.
    ///
    /// Returns:
    /// [`&String`] -> The inner value's argument name.
    pub fn name(&self) -> &String {
        match self {
            Self::String(inner) => &inner.name,
            Self::Float(inner) => &inner.name,
            Self::Integer(inner) => &inner.name,
            Self::Boolean(inner) => &inner.name,
            Self::User(inner) => &inner.name,
            Self::Channel(inner) => &inner.name,
            Self::Role(inner) => &inner.name,
        }
    }

    /// Returns the argument type of the current argument.
    ///
    /// Returns:
    /// `(ArgumentType, bool)` - The current argument's type, and whether it's optional.
    pub fn r#type(&self) -> (ArgumentType, bool) {
        match self {
            Self::String(inner) => (ArgumentType::String, inner.is_optional),
            Self::Float(inner) => (ArgumentType::Float, inner.is_optional),
            Self::Integer(inner) => (ArgumentType::Integer, inner.is_optional),
            Self::Boolean(inner) => (ArgumentType::Boolean, inner.is_optional),
            Self::User(inner) => (ArgumentType::User, inner.is_optional),
            Self::Channel(inner) => (ArgumentType::Channel, inner.is_optional),
            Self::Role(inner) => (ArgumentType::Role, inner.is_optional),
        }
    }
}

impl From<ArgumentMeta> for InnerCommandOption {
    fn from(value: ArgumentMeta) -> Self {
        match value {
            ArgumentMeta::String(inner) => inner.into(),
            ArgumentMeta::Float(inner) => inner.into(),
            ArgumentMeta::Integer(inner) => inner.into(),
            ArgumentMeta::Boolean(inner) => inner.into(),
            ArgumentMeta::User(inner) => inner.into(),
            ArgumentMeta::Channel(inner) => inner.into(),
            ArgumentMeta::Role(inner) => inner.into(),
        }
    }
}

/// Slash-command argument types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentType {
    String,
    Float,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
}

#[derive(Clone)]
pub struct StringArgument {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    min_length: Option<u16>,
    max_length: Option<u16>,

    is_optional: bool,
}

impl From<StringArgument> for InnerCommandOption {
    fn from(value: StringArgument) -> Self {
        InnerCommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: value.description,
            description_localizations: Some(value.description_i18n),
            kind: InnerCommandOptionType::String,
            min_length: value.min_length,
            max_length: value.max_length,
            min_value: None,
            max_value: None,
            name: value.name,
            name_localizations: Some(value.name_i18n),
            options: None,
            required: Some(!value.is_optional),
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

    is_optional: bool,
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
            is_optional: false,
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

    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    fn build(self) -> StringArgument {
        StringArgument {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            min_length: self.min_length,
            max_length: self.max_length,
            is_optional: self.is_optional,
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
pub struct FloatArgument {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    min_value: Option<InnerCommandOptionValue>,
    max_value: Option<InnerCommandOptionValue>,

    is_optional: bool,
}

impl From<FloatArgument> for InnerCommandOption {
    fn from(value: FloatArgument) -> Self {
        InnerCommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: value.description,
            description_localizations: Some(value.description_i18n),
            kind: InnerCommandOptionType::String,
            min_length: None,
            max_length: None,
            min_value: value.min_value,
            max_value: value.max_value,
            name: value.name,
            name_localizations: Some(value.name_i18n),
            options: None,
            required: Some(!value.is_optional),
        }
    }
}

pub struct FloatArgumentBuilder {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    min_value: Option<InnerCommandOptionValue>,
    max_value: Option<InnerCommandOptionValue>,

    is_optional: bool,
}

impl FloatArgumentBuilder {
    fn new(name: impl Into<String>) -> Self {
        FloatArgumentBuilder {
            name: name.into(),
            name_i18n: HashMap::new(),
            description: String::from("A Dyncord argument."),
            description_i18n: HashMap::new(),
            min_value: None,
            max_value: None,
            is_optional: false,
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

    pub fn min_value(mut self, value: f64) -> Self {
        self.min_value = Some(InnerCommandOptionValue::Number(value));
        self
    }

    pub fn max_value(mut self, value: f64) -> Self {
        self.max_value = Some(InnerCommandOptionValue::Number(value));
        self
    }

    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    fn build(self) -> FloatArgument {
        FloatArgument {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            min_value: self.min_value,
            max_value: self.max_value,
            is_optional: self.is_optional,
        }
    }
}

impl From<FloatArgumentBuilder> for FloatArgument {
    fn from(value: FloatArgumentBuilder) -> Self {
        value.build()
    }
}

impl From<FloatArgument> for ArgumentMeta {
    fn from(value: FloatArgument) -> Self {
        ArgumentMeta::Float(value)
    }
}

impl From<FloatArgumentBuilder> for ArgumentMeta {
    fn from(value: FloatArgumentBuilder) -> Self {
        ArgumentMeta::Float(value.build())
    }
}

#[derive(Clone)]
pub struct IntegerArgument {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    min_value: Option<InnerCommandOptionValue>,
    max_value: Option<InnerCommandOptionValue>,

    is_optional: bool,
}

impl From<IntegerArgument> for InnerCommandOption {
    fn from(value: IntegerArgument) -> Self {
        InnerCommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: value.description,
            description_localizations: Some(value.description_i18n),
            kind: InnerCommandOptionType::String,
            min_length: None,
            max_length: None,
            min_value: value.min_value,
            max_value: value.max_value,
            name: value.name,
            name_localizations: Some(value.name_i18n),
            options: None,
            required: Some(!value.is_optional),
        }
    }
}

pub struct IntegerArgumentBuilder {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    min_value: Option<InnerCommandOptionValue>,
    max_value: Option<InnerCommandOptionValue>,

    is_optional: bool,
}

impl IntegerArgumentBuilder {
    fn new(name: impl Into<String>) -> Self {
        IntegerArgumentBuilder {
            name: name.into(),
            name_i18n: HashMap::new(),
            description: String::from("A Dyncord argument."),
            description_i18n: HashMap::new(),
            min_value: None,
            max_value: None,
            is_optional: false,
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

    pub fn min_value(mut self, value: i64) -> Self {
        self.min_value = Some(InnerCommandOptionValue::Integer(value));
        self
    }

    pub fn max_value(mut self, value: i64) -> Self {
        self.max_value = Some(InnerCommandOptionValue::Integer(value));
        self
    }

    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    fn build(self) -> IntegerArgument {
        IntegerArgument {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            min_value: self.min_value,
            max_value: self.max_value,
            is_optional: self.is_optional,
        }
    }
}

impl From<IntegerArgumentBuilder> for IntegerArgument {
    fn from(value: IntegerArgumentBuilder) -> Self {
        value.build()
    }
}

impl From<IntegerArgument> for ArgumentMeta {
    fn from(value: IntegerArgument) -> Self {
        ArgumentMeta::Integer(value)
    }
}

impl From<IntegerArgumentBuilder> for ArgumentMeta {
    fn from(value: IntegerArgumentBuilder) -> Self {
        ArgumentMeta::Integer(value.build())
    }
}

#[derive(Clone)]
pub struct BooleanArgument {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    is_optional: bool,
}

impl From<BooleanArgument> for InnerCommandOption {
    fn from(value: BooleanArgument) -> Self {
        InnerCommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: value.description,
            description_localizations: Some(value.description_i18n),
            kind: InnerCommandOptionType::Boolean,
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
            name: value.name,
            name_localizations: Some(value.name_i18n),
            options: None,
            required: Some(!value.is_optional),
        }
    }
}

pub struct BooleanArgumentBuilder {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    is_optional: bool,
}

impl BooleanArgumentBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            name_i18n: HashMap::new(),
            description: String::from("A Dyncord argument."),
            description_i18n: HashMap::new(),
            is_optional: false,
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

    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    fn build(self) -> BooleanArgument {
        BooleanArgument {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            is_optional: self.is_optional,
        }
    }
}

impl From<BooleanArgumentBuilder> for BooleanArgument {
    fn from(value: BooleanArgumentBuilder) -> Self {
        value.build()
    }
}

impl From<BooleanArgument> for ArgumentMeta {
    fn from(value: BooleanArgument) -> Self {
        ArgumentMeta::Boolean(value)
    }
}

impl From<BooleanArgumentBuilder> for ArgumentMeta {
    fn from(value: BooleanArgumentBuilder) -> Self {
        ArgumentMeta::Boolean(value.build())
    }
}

#[derive(Clone)]
pub struct UserArgument {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    is_optional: bool,
}

impl From<UserArgument> for InnerCommandOption {
    fn from(value: UserArgument) -> Self {
        InnerCommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: value.description,
            description_localizations: Some(value.description_i18n),
            kind: InnerCommandOptionType::User,
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
            name: value.name,
            name_localizations: Some(value.name_i18n),
            options: None,
            required: Some(!value.is_optional),
        }
    }
}

pub struct UserArgumentBuilder {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    is_optional: bool,
}

impl UserArgumentBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            name_i18n: HashMap::new(),
            description: String::from("A Dyncord argument."),
            description_i18n: HashMap::new(),
            is_optional: false,
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

    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    fn build(self) -> UserArgument {
        UserArgument {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            is_optional: self.is_optional,
        }
    }
}

impl From<UserArgumentBuilder> for UserArgument {
    fn from(value: UserArgumentBuilder) -> Self {
        value.build()
    }
}

impl From<UserArgument> for ArgumentMeta {
    fn from(value: UserArgument) -> Self {
        ArgumentMeta::User(value)
    }
}

impl From<UserArgumentBuilder> for ArgumentMeta {
    fn from(value: UserArgumentBuilder) -> Self {
        ArgumentMeta::User(value.build())
    }
}

#[derive(Clone)]
pub struct ChannelArgument {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    is_optional: bool,
}

impl From<ChannelArgument> for InnerCommandOption {
    fn from(value: ChannelArgument) -> Self {
        InnerCommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: value.description,
            description_localizations: Some(value.description_i18n),
            kind: InnerCommandOptionType::Channel,
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
            name: value.name,
            name_localizations: Some(value.name_i18n),
            options: None,
            required: Some(!value.is_optional),
        }
    }
}

pub struct ChannelArgumentBuilder {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    is_optional: bool,
}

impl ChannelArgumentBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            name_i18n: HashMap::new(),
            description: String::from("A Dyncord argument."),
            description_i18n: HashMap::new(),
            is_optional: false,
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

    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    fn build(self) -> ChannelArgument {
        ChannelArgument {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            is_optional: self.is_optional,
        }
    }
}

impl From<ChannelArgumentBuilder> for ChannelArgument {
    fn from(value: ChannelArgumentBuilder) -> Self {
        value.build()
    }
}

impl From<ChannelArgument> for ArgumentMeta {
    fn from(value: ChannelArgument) -> Self {
        ArgumentMeta::Channel(value)
    }
}

impl From<ChannelArgumentBuilder> for ArgumentMeta {
    fn from(value: ChannelArgumentBuilder) -> Self {
        ArgumentMeta::Channel(value.build())
    }
}

#[derive(Clone)]
pub struct RoleArgument {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    is_optional: bool,
}

impl From<RoleArgument> for InnerCommandOption {
    fn from(value: RoleArgument) -> Self {
        InnerCommandOption {
            autocomplete: Some(false),
            channel_types: None,
            choices: None,
            description: value.description,
            description_localizations: Some(value.description_i18n),
            kind: InnerCommandOptionType::Role,
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
            name: value.name,
            name_localizations: Some(value.name_i18n),
            options: None,
            required: Some(!value.is_optional),
        }
    }
}

pub struct RoleArgumentBuilder {
    name: String,
    name_i18n: HashMap<String, String>,

    description: String,
    description_i18n: HashMap<String, String>,

    is_optional: bool,
}

impl RoleArgumentBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            name_i18n: HashMap::new(),
            description: String::from("A Dyncord argument."),
            description_i18n: HashMap::new(),
            is_optional: false,
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

    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    fn build(self) -> RoleArgument {
        RoleArgument {
            name: self.name,
            name_i18n: self.name_i18n,
            description: self.description,
            description_i18n: self.description_i18n,
            is_optional: self.is_optional,
        }
    }
}

impl From<RoleArgumentBuilder> for RoleArgument {
    fn from(value: RoleArgumentBuilder) -> Self {
        value.build()
    }
}

impl From<RoleArgument> for ArgumentMeta {
    fn from(value: RoleArgument) -> Self {
        ArgumentMeta::Role(value)
    }
}

impl From<RoleArgumentBuilder> for ArgumentMeta {
    fn from(value: RoleArgumentBuilder) -> Self {
        ArgumentMeta::Role(value.build())
    }
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
    /// [`(ArgumentType, bool)`] - The Discord-native type of the argument being parsed, and
    /// whether it's optional.
    fn r#type() -> (ArgumentType, bool);
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
                pinbox(Err(ArgumentError::Mistyped))
            }
        } else {
            pinbox(Err(ArgumentError::Missing))
        }
    }

    fn r#type() -> (ArgumentType, bool) {
        (ArgumentType::String, false)
    }
}

macro_rules! impl_intoargument_for_number {
    ($type:ident, $argtype:ident) => {
        impl<State> IntoArgument<State> for $type
        where
            State: StateBound,
        {
            fn into_argument_primitive(
                _ctx: SlashContext<State>,
                argument: Option<CommandDataOption>,
            ) -> DynFuture<'static, Result<Self, ArgumentError>> {
                if let Some(argument) = argument {
                    if let CommandOptionValue::Number(value) = argument.value {
                        pinbox(Self::from_f64(value).ok_or(ArgumentError::Misformatted))
                    } else {
                        pinbox(Err(ArgumentError::Mistyped))
                    }
                } else {
                    pinbox(Err(ArgumentError::Missing))
                }
            }

            fn r#type() -> (ArgumentType, bool) {
                (ArgumentType::$argtype, false)
            }
        }
    };
}

impl_intoargument_for_number!(i8, Integer);
impl_intoargument_for_number!(i16, Integer);
impl_intoargument_for_number!(i32, Integer);
impl_intoargument_for_number!(i64, Integer);
impl_intoargument_for_number!(isize, Integer);
impl_intoargument_for_number!(u8, Integer);
impl_intoargument_for_number!(u16, Integer);
impl_intoargument_for_number!(u32, Integer);
impl_intoargument_for_number!(u64, Integer);
impl_intoargument_for_number!(usize, Integer);
impl_intoargument_for_number!(f32, Float);
impl_intoargument_for_number!(f64, Float);

impl<State> IntoArgument<State> for bool
where
    State: StateBound,
{
    fn into_argument_primitive(
        _ctx: SlashContext<State>,
        argument: Option<CommandDataOption>,
    ) -> DynFuture<'static, Result<Self, ArgumentError>> {
        if let Some(argument) = argument {
            if let CommandOptionValue::Boolean(value) = argument.value {
                pinbox(Ok(value))
            } else {
                pinbox(Err(ArgumentError::Mistyped))
            }
        } else {
            pinbox(Err(ArgumentError::Missing))
        }
    }

    fn r#type() -> (ArgumentType, bool) {
        (ArgumentType::Boolean, false)
    }
}

impl<State> IntoArgument<State> for User
where
    State: StateBound,
{
    fn into_argument_primitive(
        ctx: SlashContext<State>,
        argument: Option<CommandDataOption>,
    ) -> DynFuture<'static, Result<Self, ArgumentError>> {
        if let Some(argument) = argument {
            if let CommandOptionValue::User(user_id) = argument.value {
                if let Some(resolved) = ctx.event_data.resolved {
                    if let Some(user) = resolved.users.get(&user_id) {
                        pinbox(Ok(user.clone().into()))
                    } else {
                        pinbox(Err(ArgumentError::MissingResolved))
                    }
                } else {
                    pinbox(Err(ArgumentError::MissingResolved))
                }
            } else {
                pinbox(Err(ArgumentError::Mistyped))
            }
        } else {
            pinbox(Err(ArgumentError::Missing))
        }
    }

    fn r#type() -> (ArgumentType, bool) {
        (ArgumentType::User, false)
    }
}

impl<State> IntoArgument<State> for Channel
where
    State: StateBound,
{
    fn into_argument_primitive(
        ctx: SlashContext<State>,
        argument: Option<CommandDataOption>,
    ) -> DynFuture<'static, Result<Self, ArgumentError>> {
        if let Some(argument) = argument {
            if let CommandOptionValue::Channel(channel_id) = argument.value {
                if let Some(resolved) = ctx.event_data.resolved {
                    if let Some(channel) = resolved.channels.get(&channel_id) {
                        pinbox(Ok(channel.clone().into()))
                    } else {
                        pinbox(Err(ArgumentError::MissingResolved))
                    }
                } else {
                    pinbox(Err(ArgumentError::MissingResolved))
                }
            } else {
                pinbox(Err(ArgumentError::Mistyped))
            }
        } else {
            pinbox(Err(ArgumentError::Missing))
        }
    }

    fn r#type() -> (ArgumentType, bool) {
        (ArgumentType::Channel, false)
    }
}

impl<State> IntoArgument<State> for Role
where
    State: StateBound,
{
    fn into_argument_primitive(
        ctx: SlashContext<State>,
        argument: Option<CommandDataOption>,
    ) -> DynFuture<'static, Result<Self, ArgumentError>> {
        if let Some(argument) = argument {
            if let CommandOptionValue::Role(role_id) = argument.value {
                if let Some(resolved) = ctx.event_data.resolved {
                    if let Some(role) = resolved.roles.get(&role_id) {
                        pinbox(Ok(role.clone().into()))
                    } else {
                        pinbox(Err(ArgumentError::MissingResolved))
                    }
                } else {
                    pinbox(Err(ArgumentError::MissingResolved))
                }
            } else {
                pinbox(Err(ArgumentError::Mistyped))
            }
        } else {
            pinbox(Err(ArgumentError::Missing))
        }
    }

    fn r#type() -> (ArgumentType, bool) {
        (ArgumentType::Role, false)
    }
}

impl<State, T> IntoArgument<State> for Option<T>
where
    State: StateBound,
    T: IntoArgument<State>,
{
    fn into_argument_primitive(
        ctx: SlashContext<State>,
        argument: Option<CommandDataOption>,
    ) -> DynFuture<'static, Result<Self, ArgumentError>> {
        Box::pin(async move {
            match T::into_argument_primitive(ctx, argument).await {
                Ok(value) => Ok(Some(value)),
                Err(error) => match error {
                    ArgumentError::Missing => Ok(None),
                    _ => Err(error),
                },
            }
        })
    }

    fn r#type() -> (ArgumentType, bool) {
        (T::r#type().0, true)
    }
}
