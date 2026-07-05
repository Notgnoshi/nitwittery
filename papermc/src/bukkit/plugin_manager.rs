use jni::objects::{JClass, JObject, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::{EventPriority, Plugin};
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.plugin.PluginManager`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/plugin/PluginManager.html>.
    pub PluginManager<'local> = "org/bukkit/plugin/PluginManager";
}

impl<'local> PluginManager<'local> {
    /// Mirrors `PluginManager#registerEvent(Class, Listener, EventPriority, EventExecutor, Plugin)`.
    ///
    /// `listener` and `executor` are raw handles to Java `org.bukkit.event.Listener` and
    /// `org.bukkit.plugin.EventExecutor` instances.
    #[deprecated(note = "Prefer `SetupApi::register_event` instead")]
    pub fn register_event(
        &self,
        api: &mut Api<'_, 'local>,
        event_class: &JClass<'_>,
        listener: &JObject<'_>,
        priority: EventPriority,
        executor: &JObject<'_>,
        plugin: &Plugin<'_>,
    ) -> eyre::Result<()> {
        let priority_obj = priority.as_java(api.jni())?;
        api.jni().call_method(
            &self.obj,
            jni_str!("registerEvent"),
            jni_sig!(
                "(Ljava/lang/Class;Lorg/bukkit/event/Listener;Lorg/bukkit/event/EventPriority;\
                 Lorg/bukkit/plugin/EventExecutor;Lorg/bukkit/plugin/Plugin;)V"
            ),
            &[
                JValue::Object(event_class),
                JValue::Object(listener),
                JValue::Object(&priority_obj),
                JValue::Object(executor),
                JValue::Object(plugin.as_jobject()),
            ],
        )?;
        Ok(())
    }
}
