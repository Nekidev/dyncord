//! Server member types.

#[cfg(feature = "cache-bitcode")]
use bitcode::{Decode, Encode};
#[cfg(feature = "cache-serde")]
use serde::{Deserialize, Serialize};
use twilight_model::gateway::payload::incoming::MemberUpdate as TwilightMemberUpdate;
use twilight_model::guild::{Member as TwilightMember, PartialMember as TwilightPartialMember};
use twilight_model::id::Id;

/// A Discord server member.
#[cfg_attr(feature = "cache-bitcode", derive(Encode, Decode))]
#[cfg_attr(feature = "cache-serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct Member {
    /// The user's ID.
    pub id: u64,

    /// The member's server nickname.
    pub nickname: Option<String>,

    /// IDs of the roles the member has.
    pub roles: Vec<u64>,

    pub is_muted: bool,
    pub is_deafened: bool,
    pub is_pending: bool,
}

impl Member {
    /// Updates a member from partial member data.
    ///
    /// Arguments:
    /// * `update` - The partial member data to update with.
    pub(crate) fn update_from_partial(&mut self, update: TwilightPartialMember) {
        self.nickname = update.nick;
        self.roles = update.roles.into_iter().map(Id::get).collect();
        self.is_muted = update.mute;
        self.is_deafened = update.deaf;
    }

    /// Updates a member from an update event.
    ///
    /// Arguments:
    /// * `update` - The event data to update from.
    pub(crate) fn update_from_event(&mut self, update: TwilightMemberUpdate) {
        self.nickname = update.nick;
        self.roles = update.roles.into_iter().map(Id::get).collect();
        self.is_muted = update.mute.unwrap_or(self.is_muted);
        self.is_deafened = update.deaf.unwrap_or(self.is_deafened);
        self.is_pending = update.pending;
    }
}

impl From<TwilightMember> for Member {
    fn from(value: TwilightMember) -> Self {
        Member {
            id: value.user.id.get(),
            nickname: value.nick,
            roles: value.roles.into_iter().map(Id::get).collect(),
            is_muted: value.mute,
            is_deafened: value.deaf,
            is_pending: value.pending,
        }
    }
}
