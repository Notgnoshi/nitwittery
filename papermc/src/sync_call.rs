use std::sync::{Arc, Mutex};

use jni::objects::JValue;
use jni::sys::{JNIEnv, jlong};
use jni::{Env, jni_sig, jni_str};
use tracing::warn;

use crate::api::Api;
use crate::{ctx, ffi};

/// A Rust closure invoked once on the main thread via `RustCallable.bridgeDispatch`.
///
/// The closure carries a fresh [Api] for the main thread's JNI Env, runs the user's work, and
/// writes its result into shared state captured by the closure (typically an
/// `Arc<Mutex<Option<T>>>`).
pub(crate) type SyncCallbackFn = Box<dyn for<'a, 'local> FnOnce(&mut Api<'a, 'local>) + Send>;

impl<'a, 'local> Api<'a, 'local> {
    /// Run `f` on the main server thread and block until it completes.
    ///
    /// Mirrors `org.bukkit.scheduler.BukkitScheduler#callSyncMethod(Plugin, Callable)` followed
    /// by `java.util.concurrent.Future#get()`. Intended for async event handlers (e.g.
    /// `AsyncPlayerSpawnLocationEvent`) that need to perform main-thread-only Bukkit work such
    /// as structure-locate calls that may generate chunks.
    ///
    /// Must NOT be called from the main thread itself: `Future.get()` would deadlock waiting on
    /// the scheduler to run our callable.
    ///
    /// The closure receives a fresh [Api] bound to the main thread's JNI Env. Captures must be
    /// `Send`; `JObject` references do not satisfy this and must be promoted to `Global`
    /// references before being moved across the thread boundary.
    pub fn run_sync<F, T>(&mut self, f: F) -> eyre::Result<T>
    where
        F: for<'b, 'l> FnOnce(&mut Api<'b, 'l>) -> eyre::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let result: Arc<Mutex<Option<eyre::Result<T>>>> = Arc::new(Mutex::new(None));
        let result_capture = result.clone();
        let id = ctx::next_id();
        let boxed: SyncCallbackFn = Box::new(move |api| {
            let r = f(api);
            *result_capture.lock().expect("sync_call result mutex") = Some(r);
        });
        ctx::with_ctx(|c| {
            c.sync_callbacks.insert(id, boxed);
        })
        .expect("Ctx installed during plugin_init");

        let outcome = self.run_callable_and_wait(id);
        // Drop the registry entry regardless of outcome so a panicking closure does not leak.
        ctx::with_ctx(|c| {
            c.sync_callbacks.remove(&id);
        });
        outcome?;

        result
            .lock()
            .expect("sync_call result mutex")
            .take()
            .ok_or_else(|| eyre::eyre!("sync callable returned without producing a result"))?
    }

    /// Construct a `RustCallable(id)`, schedule it via Bukkit, and block on `Future.get()`.
    fn run_callable_and_wait(&mut self, id: i64) -> eyre::Result<()> {
        let env = self.jni();
        let callable = env.new_object(
            jni_str!("io/papermc/RustCallable"),
            jni_sig!("(J)V"),
            &[JValue::Long(id)],
        )?;

        let bukkit = env
            .call_static_method(
                jni_str!("org/bukkit/Bukkit"),
                jni_str!("getScheduler"),
                jni_sig!("()Lorg/bukkit/scheduler/BukkitScheduler;"),
                &[],
            )?
            .l()?;

        let plugin =
            ctx::with_ctx(|c| c.java_plugin.clone()).expect("Ctx installed during plugin_init");

        let future = env
            .call_method(
                &bukkit,
                jni_str!("callSyncMethod"),
                jni_sig!(
                    "(Lorg/bukkit/plugin/Plugin;Ljava/util/concurrent/Callable;)\
                     Ljava/util/concurrent/Future;"
                ),
                &[JValue::Object(&plugin), JValue::Object(&callable)],
            )?
            .l()?;

        // Future.get() blocks until the main thread executes the callable. We are on a non-main
        // thread (the netty configuration thread for AsyncPlayerSpawnLocationEvent), so this is
        // safe.
        env.call_method(
            &future,
            jni_str!("get"),
            jni_sig!("()Ljava/lang/Object;"),
            &[],
        )?;
        Ok(())
    }
}

/// Trampoline target for `RustCallable.bridgeDispatch`.
///
/// papermc-loader's stable JNI symbol forwards here via the [crate::FnTable::dispatch_callable]
/// function pointer.
pub(crate) unsafe extern "C" fn dispatch_callable(env_raw: *mut JNIEnv, id: jlong) {
    let _ = ffi::bridge(env_raw, |env: &mut Env<'_>| -> eyre::Result<()> {
        let callback = ctx::with_ctx(|c| c.sync_callbacks.remove(&id)).flatten();
        let Some(callback) = callback else {
            warn!("no sync callable registered for id {id}");
            return Ok(());
        };
        let mut api = Api::new(env);
        callback(&mut api);
        Ok(())
    });
}
