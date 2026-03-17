//! Channel types.

use twilight_model::application::interaction::InteractionChannel as TwilightInteractionChannel;
use twilight_model::channel::Channel as TwilightChannel;

/// A Discord channel.
pub struct Channel {
    /// The channel's ID.
    pub id: u64,

    /// The channel's name.
    pub name: String,
}

impl From<TwilightChannel> for Channel {
    fn from(value: TwilightChannel) -> Self {
        Channel {
            id: value.id.get(),
            name: value.name.unwrap_or("Unnamed".to_string()),
        }
    }
}

impl From<TwilightInteractionChannel> for Channel {
    fn from(value: TwilightInteractionChannel) -> Self {
        Channel {
            id: value.id.get(),
            name: value.name,
        }
    }
}
