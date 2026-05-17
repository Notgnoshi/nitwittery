use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use super::DialogAction;
use crate::api::Api;
use crate::bukkit::Component;

/// Mirrors `io.papermc.paper.registry.data.dialog.ActionButton`.
///
/// See <https://jd.papermc.io/paper/1.21.11/io/papermc/paper/registry/data/dialog/ActionButton.html>.
///
/// Construct with [ActionButton::create] (mirrors the Java static factory) or via the
/// Java-side `ActionButton.builder(...)`. The Builder wrapper is deferred until a caller needs
/// finer control than `create` provides.
#[repr(transparent)]
pub struct ActionButton<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> ActionButton<'local> {
    /// Mirrors `ActionButton#create(Component, Component, int, DialogAction)`.
    ///
    /// `tooltip` and `action` correspond to `@Nullable` Java parameters; pass `None` to send
    /// null across the JNI call.
    pub fn create(
        api: &mut Api<'_, 'local>,
        label: &Component<'local>,
        tooltip: Option<&Component<'local>>,
        width: i32,
        action: Option<&DialogAction<'local>>,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let null = JObject::null();
        let tooltip_obj: &JObject<'_> = tooltip.map(|c| &c.obj).unwrap_or(&null);
        let action_obj: &JObject<'_> = action.map(|a| &a.obj).unwrap_or(&null);
        let obj = env
            .call_static_method(
                jni_str!("io/papermc/paper/registry/data/dialog/ActionButton"),
                jni_str!("create"),
                jni_sig!(
                    "(Lnet/kyori/adventure/text/Component;Lnet/kyori/adventure/text/Component;ILio/papermc/paper/registry/data/dialog/action/DialogAction;)Lio/papermc/paper/registry/data/dialog/ActionButton;"
                ),
                &[
                    JValue::Object(&label.obj),
                    JValue::Object(tooltip_obj),
                    JValue::Int(width),
                    JValue::Object(action_obj),
                ],
            )?
            .l()?;
        Ok(Self { obj })
    }
}
