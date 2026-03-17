//! Role types.

use twilight_model::guild::Role as TwilightRole;

/// A Discord role.
pub struct Role {
    /// The role's ID.
    pub id: u64,
}

impl From<TwilightRole> for Role {
    fn from(value: TwilightRole) -> Self {
        Role { id: value.id.get() }
    }
}
