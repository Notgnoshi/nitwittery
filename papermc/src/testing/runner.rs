use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::Duration;

use jni::objects::{JObject, JString};
use jni::refs::Global;
use jni::{Env, jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::{CommandSender as _, CommandSenderInst};
use crate::jobject_repr::JClassCast as _;
use crate::testing::{TESTS, TestCase, TestCtx, TestOutcome, args, battery};
use crate::{ctx, registration};

/// Register the `/test` command.
///
/// Tests are run from the main thread, but scheduled across multiple server ticks in a best-effort
/// attempt to avoid blocking the main thread. This is because a number of bukkit APIs can only be
/// called from the main thread, and I want to be able to test functionality depending on those APIs.
pub(crate) fn register_test_command(env: &mut Env<'_>) -> eyre::Result<()> {
    let id = ctx::next_id();
    ctx::with_ctx(|c| {
        c.command_handlers.insert(
            id,
            Arc::new(|env, sender_obj, args| {
                if let Err(e) = handle_test_command(env, sender_obj, args) {
                    tracing::error!("/test failed: {e:?}");
                }
                true
            }),
        );
    })
    .expect("Ctx installed during plugin_init");
    registration::register_command(env, "test", Some("papermc.test"), id)?;
    tracing::debug!("registered /test with handler id {id}");
    Ok(())
}

fn handle_test_command(
    env: &mut Env<'_>,
    sender_obj: &JObject<'_>,
    args: &[String],
) -> eyre::Result<()> {
    let plugin = plugin_name(env)?;
    let sender = CommandSenderInst::wrap_ref(env, sender_obj)?;

    let spec = match args::parse(args) {
        Ok(spec) => spec,
        Err(message) => {
            for line in message.lines() {
                emit(env, sender, &format!("[{plugin}] {line}"));
            }
            return Ok(());
        }
    };

    let mut cases: Vec<&'static TestCase> = TESTS
        .iter()
        .filter(|c| args::matches(&spec, c.name))
        .filter(|c| args::disposition(&spec, c.ignored) != args::Disposition::Exclude)
        .collect();
    cases.sort_by_key(|c| c.name);

    if spec.list {
        for case in &cases {
            emit(env, sender, &format!("[{plugin}] {}: test", case.name));
        }
        let plural = if cases.len() == 1 { "" } else { "s" };
        emit(
            env,
            sender,
            &format!("[{plugin}] {} test{plural}", cases.len()),
        );
        return Ok(());
    }

    // An empty battery is a failure: with filters it means a typo'd filter; without filters it
    // almost certainly means registry entries were stripped at link time rather than that nobody
    // wrote tests.
    if cases.is_empty() {
        if !spec.filters.is_empty() {
            emit(
                env,
                sender,
                &format!("[{plugin}] no tests matched the given filters"),
            );
        }
        let line = summary_line(&plugin, 0, 0, 0, 0, Duration::ZERO);
        emit(env, sender, &line);
        return Ok(());
    }

    let plural = if cases.len() == 1 { "" } else { "s" };
    emit(
        env,
        sender,
        &format!("[{plugin}] running {} test{plural}", cases.len()),
    );
    let run_ignored = spec.ignored || spec.include_ignored;
    if !battery::start(env, sender_obj, plugin.clone(), cases, run_ignored)? {
        emit(
            env,
            sender,
            &format!("[{plugin}] a test run is already in progress"),
        );
    }
    Ok(())
}

pub(super) fn summary_line(
    plugin: &str,
    passed: usize,
    failed: usize,
    ignored: usize,
    skipped: usize,
    elapsed: Duration,
) -> String {
    let ok = failed == 0 && passed + ignored + skipped > 0;
    let verdict = if ok { "OK" } else { "FAILED" };
    let secs = elapsed.as_secs_f64();
    format!(
        "[{plugin}] test result: {verdict}. {passed} passed; {failed} failed; {ignored} ignored; \
         {skipped} skipped; finished in {secs:.2}s"
    )
}

/// Run one test inside its own JNI local frame so its local refs are bulk-freed on exit and
/// cannot dangle into the next test. A panicking test is contained and reported as failed.
pub(super) fn run_one(
    env: &mut Env<'_>,
    sender: &Global<JObject<'static>>,
    case: &TestCase,
) -> TestOutcome {
    let result = env.with_local_frame(32, |env| -> jni::errors::Result<TestOutcome> {
        let invoker_obj = env.new_local_ref(&*sender)?;
        let mut tctx = TestCtx {
            api: Api::new(env),
            invoker: CommandSenderInst::new(invoker_obj),
        };
        Ok(
            std::panic::catch_unwind(AssertUnwindSafe(|| (case.run)(&mut tctx)))
                .unwrap_or_else(|payload| TestOutcome::Failed(panic_message(&*payload))),
        )
    });
    match result {
        Ok(outcome) => outcome,
        Err(e) => TestOutcome::Failed(format!("JNI local frame error: {e}")),
    }
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

/// Send `line` to the invoker and mirror it to the server log.
pub(super) fn emit(env: &mut Env<'_>, sender: &CommandSenderInst<'_>, line: &str) {
    tracing::info!("{line}");
    let mut api = Api::new(env);
    if let Err(e) = sender.send_plain(&mut api, line) {
        tracing::debug!("failed to send test output to invoker: {e}");
    }
}

/// The owning plugin's name from `org.bukkit.plugin.Plugin#getName()`
fn plugin_name(env: &mut Env<'_>) -> eyre::Result<String> {
    let plugin =
        ctx::with_ctx(|c| c.java_plugin.clone()).expect("Ctx installed during plugin_init");
    let name_obj = env
        .call_method(
            &*plugin,
            jni_str!("getName"),
            jni_sig!("()Ljava/lang/String;"),
            &[],
        )?
        .l()?;
    let name_jstr = env.cast_local::<JString>(name_obj)?;
    Ok(name_jstr.try_to_string(env)?)
}
