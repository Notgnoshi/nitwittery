use std::sync::{Arc, Mutex};

use jni::objects::{JObject, JValue};
use jni::refs::Global;
use jni::sys::{JNIEnv, jlong};
use jni::{Env, jni_sig, jni_str};
use tracing::warn;

use crate::api::Api;
use crate::bukkit::{Bukkit, BukkitTask};
use crate::jobject_repr::JObjectRepr as _;
use crate::{ctx, ffi};

/// A Rust closure invoked once on the main thread via `RustCallable.bridgeDispatch`.
///
/// The closure carries a fresh [Api] for the main thread's JNI Env, runs the user's work, and
/// writes its result into shared state captured by the closure (typically an
/// `Arc<Mutex<Option<T>>>`).
pub(crate) type SyncCallbackFn = Box<dyn for<'a, 'local> FnOnce(&mut Api<'a, 'local>) + Send>;

/// A repeatedly-invocable main-thread closure, for `BukkitScheduler#runTaskTimer` tasks.
pub(crate) type RepeatingCallbackFn = dyn for<'a, 'local> Fn(&mut Api<'a, 'local>) + Send + Sync;

/// A registered `RustCallable.bridgeDispatch` target.
pub(crate) enum SyncCallback {
    /// Removed from the registry by its first invocation.
    Once(SyncCallbackFn),
    /// Stays registered across invocations (repeating scheduler tasks); removed explicitly by
    /// whoever registered it.
    Repeating(Arc<RepeatingCallbackFn>),
}

/// Handle to a repeating main-thread task from [Api::schedule_repeating].
pub struct RepeatingTask {
    /// `org.bukkit.scheduler.BukkitTask` global for cancellation.
    task: Global<JObject<'static>>,
    /// Registry id of the repeating closure; removed on cancel.
    callback_id: i64,
}

impl RepeatingTask {
    /// Cancel the underlying `org.bukkit.scheduler.BukkitTask` and release the Rust closure.
    ///
    /// Safe to call from inside the repeating closure itself.
    pub fn cancel(&self, api: &mut Api<'_, '_>) -> eyre::Result<()> {
        let result = (|| -> eyre::Result<()> {
            let task_local = api.jni().new_local_ref(&self.task)?;
            let task = unsafe { BukkitTask::from_jobject(task_local) };
            task.cancel(api)
        })();
        // Drop the closure even if the Bukkit-side cancel failed, so it cannot leak.
        ctx::with_ctx(|c| {
            c.sync_callbacks.remove(&self.callback_id);
        });
        result
    }
}

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
            c.sync_callbacks.insert(id, SyncCallback::Once(boxed));
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
        let callable = new_rust_callable(self.jni(), id)?;
        let plugin = self.plugin()?;
        let scheduler = Bukkit::scheduler(self)?;
        #[allow(deprecated)]
        let future = scheduler.call_sync_method(self, &plugin, &callable)?;
        // Future.get() blocks until the main thread executes the callable. We are on a non-main
        // thread (the netty configuration thread for AsyncPlayerSpawnLocationEvent), so this is
        // safe.
        future.get(self)?;
        Ok(())
    }

    /// Schedule `f` to run repeatedly on the main server thread.
    ///
    /// Wraps `org.bukkit.scheduler.BukkitScheduler#runTaskTimer(Plugin, Runnable, long, long)` with
    /// the Rust-closure bridging handled; prefer this over the raw
    /// [BukkitScheduler::run_task_timer](crate::bukkit::BukkitScheduler::run_task_timer) wrapper.
    /// `delay_ticks` is ticks until the first run; `period_ticks` is ticks between runs. Cancel via
    /// [RepeatingTask::cancel], which also releases the closure.
    pub fn schedule_repeating<F>(
        &mut self,
        delay_ticks: i64,
        period_ticks: i64,
        f: F,
    ) -> eyre::Result<RepeatingTask>
    where
        F: for<'b, 'l> Fn(&mut Api<'b, 'l>) + Send + Sync + 'static,
    {
        let id = ctx::next_id();
        ctx::with_ctx(|c| {
            c.sync_callbacks
                .insert(id, SyncCallback::Repeating(Arc::new(f)));
        })
        .expect("Ctx installed during plugin_init");
        let result = (|| -> eyre::Result<RepeatingTask> {
            let runnable = new_rust_callable(self.jni(), id)?;
            let plugin = self.plugin()?;
            let scheduler = Bukkit::scheduler(self)?;
            #[allow(deprecated)]
            let task =
                scheduler.run_task_timer(self, &plugin, &runnable, delay_ticks, period_ticks)?;
            let task = self.jni().new_global_ref(task.as_jobject())?;
            Ok(RepeatingTask {
                task,
                callback_id: id,
            })
        })();
        // A closure whose task never got scheduled would never be invoked or cancelled; drop it.
        if result.is_err() {
            ctx::with_ctx(|c| {
                c.sync_callbacks.remove(&id);
            });
        }
        result
    }
}

/// Construct an `io.papermc.RustCallable(id)` bridge instance.
///
/// The class implements both `java.util.concurrent.Callable` and `java.lang.Runnable`, dispatching
/// either way to the [SyncCallback] registered under `id`.
fn new_rust_callable<'local>(
    env: &mut Env<'local>,
    id: i64,
) -> jni::errors::Result<JObject<'local>> {
    env.new_object(
        jni_str!("io/papermc/RustCallable"),
        jni_sig!("(J)V"),
        &[JValue::Long(id)],
    )
}

/// Trampoline target for `RustCallable.bridgeDispatch`.
///
/// papermc-loader's stable JNI symbol forwards here via the [crate::FnTable::dispatch_callable]
/// function pointer.
pub(crate) unsafe extern "C" fn dispatch_callable(env_raw: *mut JNIEnv, id: jlong) {
    let _ = ffi::bridge(env_raw, |env: &mut Env<'_>| -> eyre::Result<()> {
        // Take the callback out of the registry to invoke it without holding the Ctx lock;
        // repeating callbacks are put back for the next fire.
        let callback = ctx::with_ctx(|c| match c.sync_callbacks.remove(&id) {
            Some(SyncCallback::Once(f)) => Some(SyncCallback::Once(f)),
            Some(SyncCallback::Repeating(f)) => {
                c.sync_callbacks
                    .insert(id, SyncCallback::Repeating(f.clone()));
                Some(SyncCallback::Repeating(f))
            }
            None => None,
        })
        .flatten();
        let Some(callback) = callback else {
            warn!("no sync callable registered for id {id}");
            return Ok(());
        };
        let mut api = Api::new(env);
        match callback {
            SyncCallback::Once(f) => f(&mut api),
            SyncCallback::Repeating(f) => f(&mut api),
        }
        Ok(())
    });
}
