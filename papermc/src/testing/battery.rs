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
use crate::bukkit::mini_message::escape_tags;
use crate::bukkit::{Bukkit, CommandSenderInst};
use crate::ctx;
use crate::jobject_repr::{JClassCast as _, JObjectRepr as _};
use crate::sync_call::RepeatingTask;
use crate::testing::runner::{emit_rich, run_one, summary_line};
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
    console: Option<Arc<Global<JObject<'static>>>>, // None when 'sender' is the console.
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
    let console_inst = Bukkit::console_sender(&mut api)?;
    let console = if api
        .jni()
        .is_same_object(sender_obj, console_inst.as_jobject())?
    {
        None
    } else {
        Some(Arc::new(
            api.jni().new_global_ref(console_inst.as_jobject())?,
        ))
    };
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
            console,
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
        c.battery.as_ref().map(|b| {
            (
                b.sender.clone(),
                b.console.clone(),
                b.plugin.clone(),
                b.run_ignored,
            )
        })
    })
    .flatten();
    let Some((sender, console, plugin, run_ignored)) = header else {
        return Ok(());
    };
    let budget = Instant::now();
    loop {
        let case =
            ctx::with_ctx(|c| c.battery.as_mut().and_then(|b| b.queue.pop_front())).flatten();
        let Some(case) = case else {
            return finish(api);
        };
        run_case(api, &sender, console.as_deref(), &plugin, case, run_ignored)?;

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
    console: Option<&Global<JObject<'static>>>,
    plugin: &str,
    case: &'static TestCase,
    run_ignored: bool,
) -> eyre::Result<()> {
    let env = api.jni();
    env.with_local_frame(32, |env| -> eyre::Result<()> {
        let sender_local = env.new_local_ref(&*sender)?;
        let sender_inst = CommandSenderInst::wrap_ref(env, &sender_local)?;
        let console_local = match console {
            Some(console) => Some(env.new_local_ref(&**console)?),
            None => None,
        };
        let mut targets = vec![sender_inst];
        if let Some(console_local) = &console_local {
            targets.push(CommandSenderInst::wrap_ref(env, console_local)?);
        }
        let name = escape_tags(env, case.name)?;
        if case.ignored && !run_ignored {
            with_battery(|b| b.ignored += 1);
            let line = match case.ignore_reason {
                Some(reason) => format!(
                    "[{plugin}] test {name} ... <yellow>ignored</yellow>, {}",
                    escape_tags(env, reason)?
                ),
                None => format!("[{plugin}] test {name} ... <yellow>ignored</yellow>"),
            };
            emit_rich(env, &targets, &line);
            return Ok(());
        }
        match run_one(env, sender, case) {
            TestOutcome::Passed => {
                with_battery(|b| b.passed += 1);
                emit_rich(
                    env,
                    &targets,
                    &format!("[{plugin}] test {name} ... <green>ok</green>"),
                );
            }
            TestOutcome::Failed(message) => {
                with_battery(|b| b.failures.push((case.name, message)));
                emit_rich(
                    env,
                    &targets,
                    &format!("[{plugin}] test {name} ... <red>FAILED</red>"),
                );
            }
            TestOutcome::Skipped(reason) => {
                with_battery(|b| b.skipped += 1);
                let reason = escape_tags(env, reason)?;
                emit_rich(
                    env,
                    &targets,
                    &format!("[{plugin}] test {name} ... <yellow>skipped</yellow> ({reason})"),
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
    env.with_local_frame(32, |env| -> eyre::Result<()> {
        let sender_local = env.new_local_ref(&*battery.sender)?;
        let sender_inst = CommandSenderInst::wrap_ref(env, &sender_local)?;
        let console_local = match &battery.console {
            Some(console) => Some(env.new_local_ref(&**console)?),
            None => None,
        };
        let mut targets = vec![sender_inst];
        if let Some(console_local) = &console_local {
            targets.push(CommandSenderInst::wrap_ref(env, console_local)?);
        }
        let plugin = battery.plugin.as_str();
        if !battery.failures.is_empty() {
            emit_rich(env, &targets, &format!("[{plugin}] <red>failures:</red>"));
            for (name, message) in &battery.failures {
                let name = escape_tags(env, name)?;
                let message = escape_tags(env, message)?;
                emit_rich(env, &targets, &format!("[{plugin}] ---- {name} ----"));
                for line in message.lines() {
                    emit_rich(env, &targets, &format!("[{plugin}] {line}"));
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
        emit_rich(env, &targets, &line);
        Ok(())
    })
}
