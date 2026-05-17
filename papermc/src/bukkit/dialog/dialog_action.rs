use std::sync::Arc;

use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use super::ClickCallbackOptions;
use crate::api::Api;
use crate::bukkit::Key;
use crate::ctx;

/// Mirrors `DialogAction`.
///
/// See <https://jd.papermc.io/paper/1.21.11/io/papermc/paper/registry/data/dialog/action/DialogAction.html>.
#[repr(transparent)]
pub struct DialogAction<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> DialogAction<'local> {
    /// Mirrors `DialogAction#customClick(Key, BinaryTagHolder)`.
    ///
    /// Sends a null `BinaryTagHolder` payload; clicks land on whatever the server has registered
    /// for `key` (typically via `org.bukkit.event.player.PlayerCustomClickEvent`).
    pub fn custom_click(api: &mut Api<'_, 'local>, key: &Key<'local>) -> eyre::Result<Self> {
        let env = api.jni();
        let null_obj = JObject::null();
        let obj = env
            .call_static_method(
                jni_str!("io/papermc/paper/registry/data/dialog/action/DialogAction"),
                jni_str!("customClick"),
                jni_sig!(
                    "(Lnet/kyori/adventure/key/Key;Lnet/kyori/adventure/nbt/api/BinaryTagHolder;)Lio/papermc/paper/registry/data/dialog/action/DialogAction$CustomClickAction;"
                ),
                &[JValue::Object(&key.obj), JValue::Object(&null_obj)],
            )?
            .l()?;
        Ok(Self { obj })
    }

    /// Mirrors
    /// `DialogAction#customClick(DialogActionCallback,
    /// ClickCallback.Options)`, with the `DialogActionCallback` realized as a Rust closure routed
    /// through the `io.papermc.RustDialogActionCallback` JNI bridge.
    ///
    /// The closure receives the `DialogResponseView` and `Audience` as raw JNI objects, valid
    /// for the dispatch frame's lifetime. Wrappers for those types come later; for now callers
    /// can call into JNI directly through `api`.
    ///
    /// `options` controls how long the callback remains live and how many times it may fire;
    /// see [ClickCallbackOptions::builder].
    pub fn custom_click_callback<F>(
        api: &mut Api<'_, 'local>,
        options: &ClickCallbackOptions<'local>,
        callback: F,
    ) -> eyre::Result<Self>
    where
        F: for<'a> Fn(&mut Api<'_, 'a>, &JObject<'a>, &JObject<'a>) + Send + Sync + 'static,
    {
        let env = api.jni();
        let id = ctx::next_id();
        ctx::with_ctx(|c| {
            c.callbacks.insert(id, Arc::new(callback));
        })
        .expect("Ctx installed during plugin_init");

        let bridge = env.new_object(
            jni_str!("io/papermc/RustDialogActionCallback"),
            jni_sig!("(J)V"),
            &[JValue::Long(id)],
        )?;

        let obj = env
            .call_static_method(
                jni_str!("io/papermc/paper/registry/data/dialog/action/DialogAction"),
                jni_str!("customClick"),
                jni_sig!(
                    "(Lio/papermc/paper/registry/data/dialog/action/DialogActionCallback;Lnet/kyori/adventure/text/event/ClickCallback$Options;)Lio/papermc/paper/registry/data/dialog/action/DialogAction$CustomClickAction;"
                ),
                &[JValue::Object(&bridge), JValue::Object(&options.obj)],
            )?
            .l()?;

        Ok(Self { obj })
    }
}
