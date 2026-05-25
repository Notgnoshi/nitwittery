use jni::objects::JObject;

use crate::api::Api;

/// Mirrors `net.kyori.adventure.text.Component`.
///
/// See <https://jd.advntr.dev/api/latest/net/kyori/adventure/text/Component.html>.
///
/// This is a minimal handle; no fluent builder yet. Construct from a MiniMessage string with
/// [Component::mini_message]. A typed Component builder is a separate effort.
#[repr(transparent)]
pub struct Component<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> Component<'local> {
    /// Parse a MiniMessage string into a Component via
    /// `net.kyori.adventure.text.minimessage.MiniMessage#deserialize(String)`.
    ///
    /// See <https://docs.advntr.dev/minimessage/index.html> for tag syntax.
    pub fn mini_message(api: &mut Api<'_, 'local>, text: &str) -> eyre::Result<Self> {
        let env = api.jni();
        let obj = super::mini_message::deserialize(env, text)?;
        Ok(Self { obj })
    }
}
