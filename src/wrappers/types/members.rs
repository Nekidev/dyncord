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
}

impl Member {
    /// Updates a member from partial member data.
    ///
    /// Arguments:
    /// * `update` - The partial member data to update with.
    pub(crate) fn update_partially(&mut self, update: TwilightPartialMember) {
        self.nickname = update.nick;
        self.roles = update.roles.into_iter().map(Id::get).collect()
    }
}

impl From<TwilightMember> for Member {
    fn from(value: TwilightMember) -> Self {
        Member {
            id: value.user.id.get(),
            nickname: value.nick,
            roles: value.roles.into_iter().map(Id::get).collect(),
        }
    }
}

impl From<TwilightMemberUpdate> for Member {
    fn from(value: TwilightMemberUpdate) -> Self {
        Member {
            id: value.user.id.get(),
            nickname: value.nick,
            roles: value.roles.into_iter().map(Id::get).collect(),
        }
    }
}
