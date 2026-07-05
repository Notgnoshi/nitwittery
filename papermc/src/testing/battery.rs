//! Scheduler-driven battery runner.
//!
//! `/test` queues the matched cases in `Ctx` and schedules a repeating main-thread task via
//! `org.bukkit.scheduler.BukkitScheduler#runTaskTimer(Plugin, Runnable, long, long)`. Each fire
//! drains the queue for a bounded time budget, then yields the tick; when the queue empties the
//! task reports the summary and cancels itself.
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use jni::Env;
use jni::objects::JObject;
use jni::refs::Global;

use crate::api::Api;
use crate::bukkit::CommandSenderInst;
use crate::ctx;
use crate::jobject_repr::JClassCast as _;
use crate::sync_call::RepeatingTask;
use crate::testing::runner::{emit, run_one, summary_line};
use crate::testing::{TestCase, TestOutcome};

const TICK_BUDGET: Duration = Duration::from_millis(20);

pub(crate) struct Battery {
    queue: VecDeque<&'static TestCase>,
    run_ignored: bool,
    passed: usize,
    ignored: usize,
    skipped: usize,
    failures: Vec<(&'static str, String)>,
    start: Instant,
    /// `org.bukkit.command.CommandSender` that invoked `/test`.
    sender: Arc<Global<JObject<'static>>>,
    plugin: String,
    /// Repeating scheduler task driving [tick]; cancelled when the battery ends.
    task: RepeatingTask,
}

/// Start running a test [Battery].
///
/// Returns `Ok(false)` without starting if one is already running.
pub(super) fn start(
    env: &mut Env<'_>,
    sender_obj: &JObject<'_>,
    plugin: String,
    cases: Vec<&'static TestCase>,
    run_ignored: bool,
) -> eyre::Result<bool> {
    let already = ctx::with_ctx(|c| c.battery.is_some()).unwrap_or(true);
    if already {
        return Ok(false);
    }
    let sender = Arc::new(env.new_global_ref(sender_obj)?);
    let mut api = Api::new(env);
    let task = api.schedule_repeating(1, 1, |api| {
        if let Err(e) = tick(api) {
            tracing::error!("test battery tick failed: {e:?}");
        }
    })?;
    ctx::with_ctx(|c| {
        c.battery = Some(Battery {
            queue: cases.into(),
            run_ignored,
            passed: 0,
            ignored: 0,
            skipped: 0,
            failures: Vec::new(),
            start: Instant::now(),
            sender,
            plugin,
            task,
        });
    })
    .expect("Ctx installed during plugin_init");
    Ok(true)
}

/// Abort any in-flight battery: cancel its task and drop its state.
///
/// Tests run on the main thread, and this runs on the main thread. No tests can be executing while
/// this runs.
pub(crate) fn shutdown(env: &mut Env<'_>) {
    let battery = ctx::with_ctx(|c| c.battery.take()).flatten();
    let Some(battery) = battery else { return };
    tracing::warn!(
        "test battery aborted by plugin disable with {} tests still queued",
        battery.queue.len(),
    );
    let mut api = Api::new(env);
    if let Err(e) = battery.task.cancel(&mut api) {
        tracing::warn!("failed to cancel test battery task: {e}");
    }
}

/// Tick the test [Battery] in the [Api] context until [TICK_BUDGET] is spent or the battery ends.
fn tick(api: &mut Api<'_, '_>) -> eyre::Result<()> {
    let header = ctx::with_ctx(|c| {
        c.battery
            .as_ref()
            .map(|b| (b.sender.clone(), b.plugin.clone(), b.run_ignored))
    })
    .flatten();
    let Some((sender, plugin, run_ignored)) = header else {
        return Ok(());
    };
    let budget = Instant::now();
    loop {
        let case =
            ctx::with_ctx(|c| c.battery.as_mut().and_then(|b| b.queue.pop_front())).flatten();
        let Some(case) = case else {
            return finish(api);
        };
        run_case(api, &sender, &plugin, case, run_ignored)?;

        // We don't want to block the main thread too long, so we yield and reschedule later. This
        // doesn't prevent a single test from blocking too long, but I'm not sure there's much I can
        // do about that.
        if budget.elapsed() >= TICK_BUDGET {
            return Ok(());
        }
    }
}

/// Run or report one queued case, updating the battery counters.
fn run_case(
    api: &mut Api<'_, '_>,
    sender: &Global<JObject<'static>>,
    plugin: &str,
    case: &'static TestCase,
    run_ignored: bool,
) -> eyre::Result<()> {
    let env = api.jni();
    env.with_local_frame(16, |env| -> eyre::Result<()> {
        let sender_local = env.new_local_ref(&*sender)?;
        let sender_inst = CommandSenderInst::wrap_ref(env, &sender_local)?;
        if case.ignored && !run_ignored {
            with_battery(|b| b.ignored += 1);
            let line = match case.ignore_reason {
                Some(reason) => {
                    format!("[{plugin}] test {} ... ignored, {reason}", case.name)
                }
                None => format!("[{plugin}] test {} ... ignored", case.name),
            };
            emit(env, sender_inst, &line);
            return Ok(());
        }
        tracing::info!("test {} starting", case.name);
        match run_one(env, sender, case) {
            TestOutcome::Passed => {
                with_battery(|b| b.passed += 1);
                emit(
                    env,
                    sender_inst,
                    &format!("[{plugin}] test {} ... ok", case.name),
                );
            }
            TestOutcome::Failed(message) => {
                with_battery(|b| b.failures.push((case.name, message)));
                emit(
                    env,
                    sender_inst,
                    &format!("[{plugin}] test {} ... FAILED", case.name),
                );
            }
            TestOutcome::Skipped(reason) => {
                with_battery(|b| b.skipped += 1);
                emit(
                    env,
                    sender_inst,
                    &format!("[{plugin}] test {} ... skipped ({reason})", case.name),
                );
            }
        }
        Ok(())
    })
}

fn with_battery(f: impl FnOnce(&mut Battery)) {
    ctx::with_ctx(|c| {
        if let Some(battery) = c.battery.as_mut() {
            f(battery);
        }
    });
}

/// End the battery: cancel the task, drop the registry entry, and report the summary.
fn finish(api: &mut Api<'_, '_>) -> eyre::Result<()> {
    let battery = ctx::with_ctx(|c| c.battery.take()).flatten();
    let Some(battery) = battery else {
        return Ok(());
    };
    if let Err(e) = battery.task.cancel(api) {
        tracing::warn!("failed to cancel test battery task: {e}");
    }
    let elapsed = battery.start.elapsed();
    let env = api.jni();
    env.with_local_frame(16, |env| -> eyre::Result<()> {
        let sender_local = env.new_local_ref(&*battery.sender)?;
        let sender_inst = CommandSenderInst::wrap_ref(env, &sender_local)?;
        let plugin = battery.plugin.as_str();
        if !battery.failures.is_empty() {
            emit(env, sender_inst, &format!("[{plugin}] failures:"));
            for (name, message) in &battery.failures {
                emit(env, sender_inst, &format!("[{plugin}] ---- {name} ----"));
                for line in message.lines() {
                    emit(env, sender_inst, &format!("[{plugin}] {line}"));
                }
            }
        }
        let line = summary_line(
            plugin,
            battery.passed,
            battery.failures.len(),
            battery.ignored,
            battery.skipped,
            elapsed,
        );
        emit(env, sender_inst, &line);
        Ok(())
    })
}
