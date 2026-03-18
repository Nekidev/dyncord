//! User types.

#[cfg(feature = "cache-bitcode")]
use bitcode::{Decode, Encode};
#[cfg(feature = "cache-serde")]
use serde::{Deserialize, Serialize};
use twilight_model::channel::message::Mention as TwilightMention;
use twilight_model::user::{CurrentUser as TwilightCurrentUser, User as TwilightUser};

/// A Discord user.
#[cfg_attr(feature = "cache-bitcode", derive(Encode, Decode))]
#[cfg_attr(feature = "cache-serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct User {
    /// The user's ID.
    pub id: u64,

    /// The user's name.
    pub name: String,

    /// The user's global name, if set.
    pub name_global: Option<String>,

    /// The user's discriminator.
    ///
    /// I.e. what goes after the # in an old or application username.
    ///
    /// E.g. `MyBot#1234 -> discriminator = 1234`
    ///
    /// This value is 0 when the user has no discriminator.
    pub discriminator: u16,

    /// Whether the user is an application.
    pub is_app: bool,

    /// Whether the user has been verified.
    pub is_verified: bool,

    /// Whether the user is a Discord system application.
    pub is_system: bool,
}

impl User {
    /// Returns the name the user is displayed with around Discord.
    ///
    /// Returns:
    /// `&String` - A reference to the name.
    pub fn name_display(&self) -> &String {
        match &self.name_global {
            Some(name) => name,
            None => &self.name,
        }
    }
}

impl From<TwilightUser> for User {
    fn from(value: TwilightUser) -> Self {
        User {
            id: value.id.get(),
            name: value.name,
            name_global: value.global_name,
            discriminator: value.discriminator,
            is_app: value.bot,
            is_verified: value.verified.unwrap_or(false),
            is_system: value.system.unwrap_or(false),
        }
    }
}

impl From<TwilightCurrentUser> for User {
    fn from(value: TwilightCurrentUser) -> Self {
        Self {
            id: value.id.get(),
            name: value.name,
            name_global: value.global_name,
            discriminator: value.discriminator,
            is_app: value.bot,
            is_verified: value.verified.unwrap_or(false),
            is_system: false,
        }
    }
}

/// A Discord user mention, and the metadata sent by Discord with it.
pub struct UserMention {
    /// The user's ID.
    pub id: u64,

    /// The user's name.
    pub name: String,

    /// The user's discriminator.
    ///
    /// I.e. what goes after the # in an old or application username.
    ///
    /// E.g. `MyBot#1234 -> discriminator = 1234`
    ///
    /// This value is 0 when the user has no discriminator.
    pub discriminator: u16,

    /// Whether the user is an application.
    pub is_app: bool,
}

impl From<TwilightMention> for UserMention {
    fn from(value: TwilightMention) -> Self {
        UserMention {
            id: value.id.get(),
            name: value.name,
            discriminator: value.discriminator,
            is_app: value.bot,
        }
    }
}
