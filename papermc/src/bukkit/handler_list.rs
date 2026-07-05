use jni::objects::JValue;
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::Plugin;
use crate::jobject_repr::JObjectRepr as _;

/// Mirrors the static side of `org.bukkit.event.HandlerList`.
///
/// `papermc` unregisters the plugin's listeners automatically at disable; plugin code rarely needs
/// this.
///
/// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/event/HandlerList.html>.
pub struct HandlerList;

impl HandlerList {
    /// Mirrors `org.bukkit.event.HandlerList#unregisterAll(Plugin)`.
    pub fn unregister_all(api: &mut Api<'_, '_>, plugin: &Plugin<'_>) -> eyre::Result<()> {
        api.jni().call_static_method(
            jni_str!("org/bukkit/event/HandlerList"),
            jni_str!("unregisterAll"),
            jni_sig!("(Lorg/bukkit/plugin/Plugin;)V"),
            &[JValue::Object(plugin.as_jobject())],
        )?;
        Ok(())
    }
}
