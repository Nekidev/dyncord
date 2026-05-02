//! Server types.

#[cfg(feature = "cache-bitcode")]
use bitcode::{Decode, Encode};
#[cfg(feature = "cache-serde")]
use serde::{Deserialize, Serialize};
use twilight_model::guild::{Guild as TwilightGuild, PartialGuild as TwilightPartialGuild};

/// A Discord server.
#[cfg_attr(feature = "cache-bitcode", derive(Encode, Decode))]
#[cfg_attr(feature = "cache-serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct Server {
    /// The server's ID.
    pub id: u64,

    /// The name of the server.
    pub name: String,
}

impl Server {
    /// Updates the server from partial server data.
    ///
    /// Arguments:
    /// * `update` - The partial server data to update with.
    pub(crate) fn update_from_partial(&mut self, update: TwilightPartialGuild) {
        self.name = update.name;
    }
}

impl From<TwilightGuild> for Server {
    fn from(value: TwilightGuild) -> Self {
        Server {
            id: value.id.get(),
            name: value.name,
        }
    }
}
