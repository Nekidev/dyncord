//! Embed builders, used to create embeds to send in messages.
//!
//! See each builder's documentation for more details on how to use them. Those are:
//!
//! - [`Embed`]
//! - [`EmbedAuthor`]
//! - [`EmbedFooter`]
//! - [`EmbedImage`]
//! - [`EmbedThumbnail`]
//! - [`EmbedVideo`]
//! - [`EmbedField`]
//! - [`EmbedProvider`]
//!
//! Each of those but [`Embed`] also has a corresponding `Into*` trait (e.g. [`IntoEmbedAuthor`]
//! for [`EmbedAuthor`]) that allows you to use simple types like strings instead of having to
//! create a full builder struct.

use std::time::{self, SystemTime};

use twilight_model::channel::message::Embed as TwilightEmbed;
use twilight_model::channel::message::embed::{
    EmbedAuthor as TwilightEmbedAuthor, EmbedField as TwilightEmbedField,
    EmbedFooter as TwilightEmbedFooter, EmbedImage as TwilightEmbedImage,
    EmbedProvider as TwilightEmbedProvider, EmbedThumbnail as TwilightEmbedThumbnail,
    EmbedVideo as TwilightEmbedVideo,
};
pub use twilight_model::util::Timestamp;

/// An embed builder, used to create embeds to send in messages.
///
/// Internally, it wraps the [`Embed`](TwilightEmbed) struct from the Twilight library. However,
/// since that type is not pretty to create, this builder provides a more ergonomic interface for
/// creating embeds. You can use either type to send embeds in messages.
///
/// ```
/// use dyncord::builders::embeds::Embed;
/// ctx.send("").embed(Embed::build().title("Hello world!")).await?;
///
/// // or
///
/// use twilight_model::channel::message::Embed;
/// ctx.send("").embed(Embed { ... }).await?;
/// ```
///
/// Some of the methods the embed builder has support many different types representing the value
/// of that field. For example, the [`Embed::author()`] field can be set with either:
///
/// ```
/// Embed::build().author("Author Name");
/// // or
/// Embed::build().author(EmbedAuthor::new("Author Name"));
/// // or
/// Embed::build().author(TwilightEmbedAuthor { ... });
/// ```
///
/// This is true for the following methods:
///
/// - [`Embed::author()`]
/// - [`Embed::footer()`]
/// - [`Embed::image()`]
/// - [`Embed::thumbnail()`]
/// - [`Embed::video()`]
/// - [`Embed::field()`]
/// - [`Embed::provider()`]
///
/// Check each method's documentation for more details on what types are supported for that method.
#[derive(Default, Clone)]
pub struct Embed {
    title: Option<String>,
    description: Option<String>,
    color: Option<u32>,
    url: Option<String>,
    author: Option<TwilightEmbedAuthor>,
    footer: Option<TwilightEmbedFooter>,
    image: Option<TwilightEmbedImage>,
    thumbnail: Option<TwilightEmbedThumbnail>,
    video: Option<TwilightEmbedVideo>,
    fields: Vec<TwilightEmbedField>,
    timestamp: Option<Timestamp>,
    provider: Option<TwilightEmbedProvider>,
}

impl Embed {
    /// Creates a new instance of [`Embed`].
    ///
    /// Returns:
    /// [`Embed`] - A new instance of the embed builder.
    pub fn build() -> Self {
        Self::default()
    }

    /// Sets the title of the embed.
    ///
    /// Arguments:
    /// * `title` - The title of the embed.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the title set.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description of the embed.
    ///
    /// Arguments:
    /// * `description` - The description of the embed.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the description set.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the color of the embed.
    ///
    /// Arguments:
    /// * `color` - The color of the embed, as a hexadecimal RGB value (e.g. `0xFF0000` for red).
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the color set.
    pub fn color(mut self, color: u32) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the URL of the embed.
    ///
    /// Arguments:
    /// * `url` - The URL of the embed, which makes the title of the embed a hyperlink to this URL.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the URL set.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the author of the embed.
    ///
    /// Arguments:
    /// * `author` - The author of the embed, which displays a small name and icon at the top of
    ///   the embed. It can be either a string representing the author's name, an [`EmbedAuthor`]
    ///   builder, or a [`TwilightEmbedAuthor`] struct.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the author set.
    pub fn author(mut self, author: impl IntoEmbedAuthor) -> Self {
        self.author = Some(author.into_embed_author());
        self
    }

    /// Sets the footer of the embed.
    ///
    /// Arguments:
    /// * `footer` - The footer of the embed, which displays a small name and icon at the bottom of
    ///   the embed. It can be either a string representing the footer's text, an [`EmbedFooter`]
    ///   builder, or a [`TwilightEmbedFooter`] struct.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the footer set.
    pub fn footer(mut self, footer: impl IntoEmbedFooter) -> Self {
        self.footer = Some(footer.into_embed_footer());
        self
    }

    /// Sets the image of the embed.
    ///
    /// Arguments:
    /// * `image` - The image of the embed, which displays a large image in the embed. It can be
    ///   either a string representing the URL of the image, an [`EmbedImage`] builder, or a
    ///   [`TwilightEmbedImage`] struct.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the image set.
    pub fn image(mut self, image: impl IntoEmbedImage) -> Self {
        self.image = Some(image.into_embed_image());
        self
    }

    /// Sets the thumbnail of the embed.
    ///
    /// Arguments:
    /// * `thumbnail` - The thumbnail of the embed, which displays a small image in the embed. It
    ///   can be either a string representing the URL of the thumbnail, an [`EmbedThumbnail`]
    ///   builder, or a [`TwilightEmbedThumbnail`] struct.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the thumbnail set.
    pub fn thumbnail(mut self, thumbnail: impl IntoEmbedThumbnail) -> Self {
        self.thumbnail = Some(thumbnail.into_embed_thumbnail());
        self
    }

    /// Sets the video of the embed.
    ///
    /// Arguments:
    /// * `video` - The video of the embed, which displays a video in the embed. It can be either a
    ///   string representing the URL of the video, an [`EmbedVideo`] builder, or a
    ///   [`TwilightEmbedVideo`] struct.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the video set.
    pub fn video(mut self, video: impl IntoEmbedVideo) -> Self {
        self.video = Some(video.into_embed_video());
        self
    }

    /// Adds a field to the embed.
    ///
    /// Arguments:
    /// * `field` - The field to add to the embed. It can be either an [`EmbedField`] builder or a
    ///   [`TwilightEmbedField`] struct.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the field added.
    pub fn field(mut self, field: impl IntoEmbedField) -> Self {
        self.fields.push(field.into_embed_field());
        self
    }

    /// Sets the timestamp of the embed.
    ///
    /// See [`Embed::timestamp_now()`] if you want to set the timestamp to the current date and
    /// time.
    ///
    /// Arguments:
    /// * `timestamp` - The timestamp of the embed, which displays a timestamp at the bottom of the
    ///   embed.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the timestamp set.
    pub fn timestamp(mut self, timestamp: impl Into<Timestamp>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Sets the timestamp to the current date and time.
    ///
    /// This is a convenience method that sets the timestamp to the current date and time, so you
    /// don't have to manually get the current time and convert it to a [`Timestamp`]. It is
    /// equivalent to calling `embed.timestamp(Timestamp::from_secs(current_time_in_seconds))`, but
    /// more ergonomic.
    ///
    /// See [`Embed::timestamp()`] if you want to set the timestamp to a specific value instead of
    /// the current date and time.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the timestamp set to the current date and time.
    pub fn timestamp_now(self) -> Self {
        let current_secs = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        self.timestamp(Timestamp::from_secs(current_secs as i64).expect("Now is invalid?"))
    }

    /// Sets the provider of the embed.
    ///
    /// Arguments:
    /// * `provider` - The provider of the embed, which displays a small name and icon at the top
    ///   of the embed, next to the author. It can be either a string representing the provider's
    ///   name, an [`EmbedProvider`] builder, or a [`TwilightEmbedProvider`] struct.
    ///
    /// Returns:
    /// [`Embed`] - The embed builder with the provider set.
    pub fn provider(mut self, provider: impl IntoEmbedProvider) -> Self {
        self.provider = Some(provider.into_embed_provider());
        self
    }
}

impl From<Embed> for TwilightEmbed {
    fn from(from: Embed) -> TwilightEmbed {
        TwilightEmbed {
            title: from.title,
            author: from.author,
            color: from.color,
            description: from.description,
            fields: from.fields,
            footer: from.footer,
            image: from.image,
            thumbnail: from.thumbnail,
            video: from.video,
            provider: None,
            timestamp: from.timestamp,
            kind: "".into(),
            url: from.url,
        }
    }
}

/// An embed author builder, used to create the author field of an embed.
pub struct EmbedAuthor {
    name: String,
    url: Option<String>,
    icon_url: Option<String>,
    proxy_icon_url: Option<String>,
}

impl EmbedAuthor {
    /// Creates a new instance of [`EmbedAuthor`].
    ///
    /// Arguments:
    /// * `name` - The name of the author.
    ///
    /// Returns:
    /// [`EmbedAuthor`] - A new instance of the embed author builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: None,
            icon_url: None,
            proxy_icon_url: None,
        }
    }

    /// Sets the URL of the author.
    ///
    /// Arguments:
    /// * `url` - The URL of the author, which makes the author's name a hyperlink to this URL.
    ///
    /// Returns:
    /// [`EmbedAuthor`] - The embed author builder with the URL set.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the icon URL of the author.
    ///
    /// Arguments:
    /// * `icon_url` - The URL of the author's icon, which displays a small image next to the
    ///   author's name.
    ///
    /// Returns:
    /// [`EmbedAuthor`] - The embed author builder with the icon URL set.
    pub fn icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    /// Sets the proxy icon URL of the author.
    ///
    /// Arguments:
    /// * `proxy_icon_url` - The proxied URL of the author's icon.
    ///
    /// Returns:
    /// [`EmbedAuthor`] - The embed author builder with the proxy icon URL set.
    pub fn proxy_icon_url(mut self, proxy_icon_url: impl Into<String>) -> Self {
        self.proxy_icon_url = Some(proxy_icon_url.into());
        self
    }
}

/// Converts a type into an [`TwilightEmbedAuthor`].
///
/// This is used to allow for more ergonomic creation of embed authors, by allowing you to use
/// simple types like strings instead of having to create a full [`TwilightEmbedAuthor`] struct.
pub trait IntoEmbedAuthor {
    /// Converts this type into a [`TwilightEmbedAuthor`].
    ///
    /// Returns:
    /// [`TwilightEmbedAuthor`] - The converted embed author.
    fn into_embed_author(self) -> TwilightEmbedAuthor;
}

impl IntoEmbedAuthor for EmbedAuthor {
    fn into_embed_author(self) -> TwilightEmbedAuthor {
        TwilightEmbedAuthor {
            name: self.name,
            url: self.url,
            icon_url: self.icon_url,
            proxy_icon_url: self.proxy_icon_url,
        }
    }
}

impl IntoEmbedAuthor for TwilightEmbedAuthor {
    fn into_embed_author(self) -> TwilightEmbedAuthor {
        self
    }
}

impl IntoEmbedAuthor for String {
    fn into_embed_author(self) -> TwilightEmbedAuthor {
        TwilightEmbedAuthor {
            name: self,
            url: None,
            icon_url: None,
            proxy_icon_url: None,
        }
    }
}

impl IntoEmbedAuthor for &str {
    fn into_embed_author(self) -> TwilightEmbedAuthor {
        TwilightEmbedAuthor {
            name: self.to_string(),
            url: None,
            icon_url: None,
            proxy_icon_url: None,
        }
    }
}

/// An embed footer builder, used to create the footer field of an embed.
///
/// The footer is a small section at the bottom of the embed, which can contain text and an icon.
/// It is often used to display additional information about the embed, such as the time it was
/// created or the name of the bot that sent it.
pub struct EmbedFooter {
    text: String,
    icon_url: Option<String>,
    proxy_icon_url: Option<String>,
}

impl EmbedFooter {
    /// Creates a new instance of [`EmbedFooter`].
    ///
    /// Arguments:
    /// * `text` - The text of the footer.
    ///
    /// Returns:
    /// [`EmbedFooter`] - A new instance of the embed footer builder.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon_url: None,
            proxy_icon_url: None,
        }
    }

    /// Sets the icon URL of the footer.
    ///
    /// Arguments:
    /// * `icon_url` - The URL of the footer's icon, which displays a small image next to the
    ///   footer's text.
    ///
    /// Returns:
    /// [`EmbedFooter`] - The embed footer builder with the icon URL set.
    pub fn icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    /// Sets the proxy icon URL of the footer.
    ///
    /// Arguments:
    /// * `proxy_icon_url` - The proxied URL of the footer's icon.
    ///
    /// Returns:
    /// [`EmbedFooter`] - The embed footer builder with the proxy icon URL set.
    pub fn proxy_icon_url(mut self, proxy_icon_url: impl Into<String>) -> Self {
        self.proxy_icon_url = Some(proxy_icon_url.into());
        self
    }
}

/// Converts a type into an [`TwilightEmbedFooter`].
///
/// This is used to allow for more ergonomic creation of embed footers, by allowing you to use
/// simple types like strings instead of having to create a full [`TwilightEmbedFooter`] struct.
pub trait IntoEmbedFooter {
    /// Converts this type into a [`TwilightEmbedFooter`].
    ///
    /// Returns:
    /// [`TwilightEmbedFooter`] - The converted embed footer.
    fn into_embed_footer(self) -> TwilightEmbedFooter;
}

impl IntoEmbedFooter for EmbedFooter {
    fn into_embed_footer(self) -> TwilightEmbedFooter {
        TwilightEmbedFooter {
            text: self.text,
            icon_url: self.icon_url,
            proxy_icon_url: self.proxy_icon_url,
        }
    }
}

impl IntoEmbedFooter for TwilightEmbedFooter {
    fn into_embed_footer(self) -> TwilightEmbedFooter {
        self
    }
}

impl IntoEmbedFooter for String {
    fn into_embed_footer(self) -> TwilightEmbedFooter {
        TwilightEmbedFooter {
            text: self,
            icon_url: None,
            proxy_icon_url: None,
        }
    }
}

impl IntoEmbedFooter for &str {
    fn into_embed_footer(self) -> TwilightEmbedFooter {
        TwilightEmbedFooter {
            text: self.to_string(),
            icon_url: None,
            proxy_icon_url: None,
        }
    }
}

/// An embed image builder, used to create the image field of an embed.
pub struct EmbedImage {
    url: String,
    proxy_url: Option<String>,
    height: Option<u64>,
    width: Option<u64>,
}

impl EmbedImage {
    /// Creates a new instance of [`EmbedImage`].
    ///
    /// Arguments:
    /// * `url` - The URL of the image to display in the embed.
    ///
    /// Returns:
    /// [`EmbedImage`] - A new instance of the embed image builder.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
        }
    }

    /// Sets the proxy URL of the image.
    ///
    /// Arguments:
    /// * `proxy_url` - The proxied URL of the image.
    ///
    /// Returns:
    /// [`EmbedImage`] - The embed image builder with the proxy URL set.
    pub fn proxy_url(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy_url = Some(proxy_url.into());
        self
    }

    /// Sets the height of the image.
    ///
    /// Arguments:
    /// * `height` - The height of the image in pixels.
    ///
    /// Returns:
    /// [`EmbedImage`] - The embed image builder with the height set.
    pub fn height(mut self, height: u64) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the width of the image.
    ///
    /// Arguments:
    /// * `width` - The width of the image in pixels.
    ///
    /// Returns:
    /// [`EmbedImage`] - The embed image builder with the width set.
    pub fn width(mut self, width: u64) -> Self {
        self.width = Some(width);
        self
    }
}

/// Converts a type into a [`TwilightEmbedImage`].
///
/// This is used to allow for more ergonomic creation of embed images, by allowing you to use
/// simple types like strings instead of having to create a full [`TwilightEmbedImage`] struct.
pub trait IntoEmbedImage {
    /// Converts this type into a [`TwilightEmbedImage`].
    ///
    /// Returns:
    /// [`TwilightEmbedImage`] - The converted embed image.
    fn into_embed_image(self) -> TwilightEmbedImage;
}

impl IntoEmbedImage for EmbedImage {
    fn into_embed_image(self) -> TwilightEmbedImage {
        TwilightEmbedImage {
            url: self.url,
            proxy_url: self.proxy_url,
            height: self.height,
            width: self.width,
        }
    }
}

impl IntoEmbedImage for TwilightEmbedImage {
    fn into_embed_image(self) -> TwilightEmbedImage {
        self
    }
}

impl IntoEmbedImage for String {
    fn into_embed_image(self) -> TwilightEmbedImage {
        TwilightEmbedImage {
            url: self,
            proxy_url: None,
            height: None,
            width: None,
        }
    }
}

impl IntoEmbedImage for &str {
    fn into_embed_image(self) -> TwilightEmbedImage {
        TwilightEmbedImage {
            url: self.to_string(),
            proxy_url: None,
            height: None,
            width: None,
        }
    }
}

/// An embed thumbnail builder, used to create the thumbnail field of an embed.
///
/// The thumbnail is a small image that appears in the top right corner of the embed. It is often
/// used to display a small version of the main image of the embed, or to display an icon
/// representing the content of the embed.
pub struct EmbedThumbnail {
    url: String,
    proxy_url: Option<String>,
    height: Option<u64>,
    width: Option<u64>,
}

impl EmbedThumbnail {
    /// Creates a new instance of [`EmbedThumbnail`].
    ///
    /// Arguments:
    /// * `url` - The URL of the thumbnail to display in the embed.
    ///
    /// Returns:
    /// [`EmbedThumbnail`] - A new instance of the embed thumbnail builder.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
        }
    }

    /// Sets the proxy URL of the thumbnail.
    ///
    /// Arguments:
    /// * `proxy_url` - The proxied URL of the thumbnail.
    ///
    /// Returns:
    /// [`EmbedThumbnail`] - The embed thumbnail builder with the proxy URL set.
    pub fn proxy_url(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy_url = Some(proxy_url.into());
        self
    }

    /// Sets the height of the thumbnail.
    ///
    /// Arguments:
    /// * `height` - The height of the thumbnail in pixels.
    ///
    /// Returns:
    /// [`EmbedThumbnail`] - The embed thumbnail builder with the height set.
    pub fn height(mut self, height: u64) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the width of the thumbnail.
    ///
    /// Arguments:
    /// * `width` - The width of the thumbnail in pixels.
    ///
    /// Returns:
    /// [`EmbedThumbnail`] - The embed thumbnail builder with the width set.
    pub fn width(mut self, width: u64) -> Self {
        self.width = Some(width);
        self
    }
}

/// Converts a type into a [`TwilightEmbedThumbnail`].
///
/// This is used to allow for more ergonomic creation of embed thumbnails, by allowing you to use
/// simple types like strings instead of having to create a full [`TwilightEmbedThumbnail`] struct.
pub trait IntoEmbedThumbnail {
    /// Converts this type into a [`TwilightEmbedThumbnail`].
    ///
    /// Returns:
    /// [`TwilightEmbedThumbnail`] - The converted embed thumbnail.
    fn into_embed_thumbnail(self) -> TwilightEmbedThumbnail;
}

impl IntoEmbedThumbnail for EmbedThumbnail {
    fn into_embed_thumbnail(self) -> TwilightEmbedThumbnail {
        TwilightEmbedThumbnail {
            url: self.url,
            proxy_url: self.proxy_url,
            height: self.height,
            width: self.width,
        }
    }
}

impl IntoEmbedThumbnail for TwilightEmbedThumbnail {
    fn into_embed_thumbnail(self) -> TwilightEmbedThumbnail {
        self
    }
}

impl IntoEmbedThumbnail for String {
    fn into_embed_thumbnail(self) -> TwilightEmbedThumbnail {
        TwilightEmbedThumbnail {
            url: self,
            proxy_url: None,
            height: None,
            width: None,
        }
    }
}

impl IntoEmbedThumbnail for &str {
    fn into_embed_thumbnail(self) -> TwilightEmbedThumbnail {
        TwilightEmbedThumbnail {
            url: self.to_string(),
            proxy_url: None,
            height: None,
            width: None,
        }
    }
}

/// An embed video builder, used to create the video field of an embed.
///
/// The video is a video that appears in the embed. It is often used to display a video related to
/// the content of the embed, such as a YouTube video or a Twitch stream.
pub struct EmbedVideo {
    url: String,
    proxy_url: Option<String>,
    height: Option<u64>,
    width: Option<u64>,
}

impl EmbedVideo {
    /// Creates a new instance of [`EmbedVideo`].
    ///
    /// Arguments:
    /// * `url` - The URL of the video to display in the embed.
    ///
    /// Returns:
    /// [`EmbedVideo`] - A new instance of the embed video builder.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
        }
    }

    /// Sets the proxy URL of the video.
    ///
    /// Arguments:
    /// * `proxy_url` - The proxied URL of the video.
    ///
    /// Returns:
    /// [`EmbedVideo`] - The embed video builder with the proxy URL set.
    pub fn proxy_url(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy_url = Some(proxy_url.into());
        self
    }

    /// Sets the height of the video.
    ///
    /// Arguments:
    /// * `height` - The height of the video in pixels.
    ///
    /// Returns:
    /// [`EmbedVideo`] - The embed video builder with the height set.
    pub fn height(mut self, height: u64) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the width of the video.
    ///
    /// Arguments:
    /// * `width` - The width of the video in pixels.
    ///
    /// Returns:
    /// [`EmbedVideo`] - The embed video builder with the width set.
    pub fn width(mut self, width: u64) -> Self {
        self.width = Some(width);
        self
    }
}

/// Converts a type into a [`TwilightEmbedVideo`].
///
/// This is used to allow for more ergonomic creation of embed videos, by allowing you to use
/// simple types like strings instead of having to create a full [`TwilightEmbedVideo`] struct.
pub trait IntoEmbedVideo {
    /// Converts this type into a [`TwilightEmbedVideo`].
    ///
    /// Returns:
    /// [`TwilightEmbedVideo`] - The converted embed video.
    fn into_embed_video(self) -> TwilightEmbedVideo;
}

impl IntoEmbedVideo for EmbedVideo {
    fn into_embed_video(self) -> TwilightEmbedVideo {
        TwilightEmbedVideo {
            url: Some(self.url),
            proxy_url: self.proxy_url,
            height: self.height,
            width: self.width,
        }
    }
}

impl IntoEmbedVideo for TwilightEmbedVideo {
    fn into_embed_video(self) -> TwilightEmbedVideo {
        self
    }
}

impl IntoEmbedVideo for String {
    fn into_embed_video(self) -> TwilightEmbedVideo {
        TwilightEmbedVideo {
            url: Some(self),
            proxy_url: None,
            height: None,
            width: None,
        }
    }
}

impl IntoEmbedVideo for &str {
    fn into_embed_video(self) -> TwilightEmbedVideo {
        TwilightEmbedVideo {
            url: Some(self.to_string()),
            proxy_url: None,
            height: None,
            width: None,
        }
    }
}

/// An embed field builder, used to create the fields of an embed.
///
/// Fields are small sections that appear in the middle of the embed, which can contain a name and
/// a value. They are often used to display additional information about the content of the embed,
/// such as a list of items or a set of key-value pairs.
pub struct EmbedField {
    name: String,
    value: String,
    is_inline: bool,
}

impl EmbedField {
    /// Creates a new instance of [`EmbedField`].
    ///
    /// Arguments:
    /// * `name` - The name of the field.
    /// * `value` - The value of the field.
    ///
    /// Returns:
    /// [`EmbedField`] - A new instance of the embed field builder.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            is_inline: false,
        }
    }

    /// Sets whether the field is inline or not.
    ///
    /// Arguments:
    /// * `is_inline` - Whether the field should be displayed inline with other fields or on its
    ///   own line.
    ///
    /// Returns:
    /// [`EmbedField`] - The embed field builder with the inline setting set.
    pub fn inline(mut self, is_inline: bool) -> Self {
        self.is_inline = is_inline;
        self
    }
}

/// Converts a type into a [`TwilightEmbedField`].
///
/// This is used to allow for more ergonomic creation of embed fields, by allowing you to use
/// simple types like strings instead of having to create a full [`TwilightEmbedField`] struct.
pub trait IntoEmbedField {
    /// Converts this type into a [`TwilightEmbedField`].
    ///
    /// Returns:
    /// [`TwilightEmbedField`] - The converted embed field.
    fn into_embed_field(self) -> TwilightEmbedField;
}

impl IntoEmbedField for EmbedField {
    fn into_embed_field(self) -> TwilightEmbedField {
        TwilightEmbedField {
            name: self.name,
            value: self.value,
            inline: self.is_inline,
        }
    }
}

impl IntoEmbedField for TwilightEmbedField {
    fn into_embed_field(self) -> TwilightEmbedField {
        self
    }
}

/// An embed provider builder, used to create the provider field of an embed.
///
/// The provider is a small section that appears at the top of the embed, which can contain a name
/// and a URL. It is often used to display the name of the service that provided the content of the
/// embed, such as YouTube or Twitch.
pub struct EmbedProvider {
    name: String,
    url: Option<String>,
}

impl EmbedProvider {
    /// Creates a new instance of [`EmbedProvider`].
    ///
    /// Arguments:
    /// * `name` - The name of the provider.
    ///
    /// Returns:
    /// [`EmbedProvider`] - A new instance of the embed provider builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: None,
        }
    }

    /// Sets the URL of the provider.
    ///
    /// Arguments:
    /// * `url` - The URL of the provider, which makes the provider's name a hyperlink to this URL.
    ///
    /// Returns:
    /// [`EmbedProvider`] - The embed provider builder with the URL set.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

/// Converts a type into a [`TwilightEmbedProvider`].
///
/// This is used to allow for more ergonomic creation of embed providers, by allowing you to use
/// simple types like strings instead of having to create a full [`TwilightEmbedProvider`] struct.
pub trait IntoEmbedProvider {
    /// Converts this type into a [`TwilightEmbedProvider`].
    ///
    /// Returns:
    /// [`TwilightEmbedProvider`] - The converted embed provider.
    fn into_embed_provider(self) -> TwilightEmbedProvider;
}

impl IntoEmbedProvider for EmbedProvider {
    fn into_embed_provider(self) -> TwilightEmbedProvider {
        TwilightEmbedProvider {
            name: Some(self.name),
            url: self.url,
        }
    }
}

impl IntoEmbedProvider for TwilightEmbedProvider {
    fn into_embed_provider(self) -> TwilightEmbedProvider {
        self
    }
}

impl IntoEmbedProvider for String {
    fn into_embed_provider(self) -> TwilightEmbedProvider {
        TwilightEmbedProvider {
            name: Some(self),
            url: None,
        }
    }
}

impl IntoEmbedProvider for &str {
    fn into_embed_provider(self) -> TwilightEmbedProvider {
        TwilightEmbedProvider {
            name: Some(self.to_string()),
            url: None,
        }
    }
}
