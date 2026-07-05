use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::Plugin;
use crate::java::Future;
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.scheduler.BukkitScheduler`.
    ///
    /// Obtain via [crate::bukkit::Bukkit::scheduler]. For scheduling main-thread work in Rust,
    /// prefer [Api::run_sync] and [Api::schedule_repeating], which handle the Java `Callable` /
    /// `Runnable` bridging.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/scheduler/BukkitScheduler.html>.
    pub BukkitScheduler<'local> = "org/bukkit/scheduler/BukkitScheduler";
}

impl<'local> BukkitScheduler<'local> {
    /// Mirrors `BukkitScheduler#runTaskTimer(Plugin, Runnable, long, long)`.
    ///
    /// `runnable` is a raw handle to a `java.lang.Runnable`; prefer [Api::schedule_repeating] for
    /// Rust closures. Delay and period are in server ticks.
    #[deprecated(note = "Prefer `Api::schedule_repeating` instead")]
    pub fn run_task_timer(
        &self,
        api: &mut Api<'_, 'local>,
        plugin: &Plugin<'_>,
        runnable: &JObject<'_>,
        delay_ticks: i64,
        period_ticks: i64,
    ) -> eyre::Result<BukkitTask<'local>> {
        let obj = api
            .jni()
            .call_method(
                &self.obj,
                jni_str!("runTaskTimer"),
                jni_sig!(
                    "(Lorg/bukkit/plugin/Plugin;Ljava/lang/Runnable;JJ)\
                     Lorg/bukkit/scheduler/BukkitTask;"
                ),
                &[
                    JValue::Object(plugin.as_jobject()),
                    JValue::Object(runnable),
                    JValue::Long(delay_ticks),
                    JValue::Long(period_ticks),
                ],
            )?
            .l()?;
        Ok(unsafe { BukkitTask::from_jobject(obj) })
    }

    /// Mirrors `BukkitScheduler#callSyncMethod(Plugin, Callable)`.
    ///
    /// `callable` is a raw handle to a `java.util.concurrent.Callable`; prefer [Api::run_sync] for
    /// Rust closures.
    #[deprecated(note = "Prefer `Api::run_sync` instead")]
    pub fn call_sync_method(
        &self,
        api: &mut Api<'_, 'local>,
        plugin: &Plugin<'_>,
        callable: &JObject<'_>,
    ) -> eyre::Result<Future<'local>> {
        let obj = api
            .jni()
            .call_method(
                &self.obj,
                jni_str!("callSyncMethod"),
                jni_sig!(
                    "(Lorg/bukkit/plugin/Plugin;Ljava/util/concurrent/Callable;)\
                     Ljava/util/concurrent/Future;"
                ),
                &[
                    JValue::Object(plugin.as_jobject()),
                    JValue::Object(callable),
                ],
            )?
            .l()?;
        Ok(unsafe { Future::from_jobject(obj) })
    }
}

papermc_jobject! {
    /// Mirrors `org.bukkit.scheduler.BukkitTask`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/scheduler/BukkitTask.html>.
    pub BukkitTask<'local> = "org/bukkit/scheduler/BukkitTask";
}

impl<'local> BukkitTask<'local> {
    /// Mirrors `org.bukkit.scheduler.BukkitTask#cancel()`.
    pub fn cancel(&self, api: &mut Api<'_, 'local>) -> eyre::Result<()> {
        api.jni()
            .call_method(&self.obj, jni_str!("cancel"), jni_sig!("()V"), &[])?;
        Ok(())
    }
}
