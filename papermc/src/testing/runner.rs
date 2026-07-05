use std::panic::AssertUnwindSafe;
use std::time::Duration;

use jni::Env;
use jni::objects::JObject;
use jni::refs::Global;

use crate::api::Api;
use crate::bukkit::{CommandSender as _, CommandSenderInst};
use crate::jobject_repr::{JClassCast as _, JObjectRepr as _};
use crate::plugin::Plugin;
use crate::setup_api::{Completer, SetupApi};
use crate::testing::{TESTS, TestCase, TestCtx, TestOutcome, args, battery};

/// Register the `/test` command through the public [SetupApi], like any plugin command.
///
/// Tests are run from the main thread, but scheduled across multiple server ticks in a best-effort
/// attempt to avoid blocking the main thread. This is because a number of bukkit APIs can only be
/// called from the main thread, and I want to be able to test functionality depending on those APIs.
pub(crate) fn register_test_command<P: Plugin>(
    setup: &mut SetupApi<'_, '_, P>,
) -> eyre::Result<()> {
    let completer: Completer<P> = Box::new(|_plugin, _api, _sender, args| {
        let current = args.last().map(String::as_str).unwrap_or("");
        Ok(args::complete(current, TESTS.iter().map(|c| c.name)))
    });
    setup.register_command(
        "test",
        Some("papermc.test"),
        Some(completer),
        |_plugin, api, sender, args| {
            handle_test_command(api.jni(), sender.as_jobject(), args)?;
            Ok(true)
        },
    )
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
        emit_rich(env, &[sender], &line);
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

/// Build the summary line, its verdict colored to match libtest: green `OK`, red `FAILED`.
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
    let color = if ok { "green" } else { "red" };
    let secs = elapsed.as_secs_f64();
    format!(
        "[{plugin}] test result: <{color}>{verdict}</{color}>. {passed} passed; {failed} failed; \
         {ignored} ignored; {skipped} skipped; finished in {secs:.2}s"
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

/// Send literal `line` to the invoker
pub(super) fn emit(env: &mut Env<'_>, sender: &CommandSenderInst<'_>, line: &str) {
    let mut api = Api::new(env);
    if let Err(e) = sender.send_plain(&mut api, line) {
        tracing::debug!("failed to send test output to invoker: {e}");
    }
}

/// Send a MiniMessage-formatted `line` to every target.
///
/// Battery output goes to the invoker and, when the invoker is not the console, to the console too.
/// Dynamic content inside `line` must be escaped via
/// [escape_tags](crate::bukkit::mini_message::escape_tags).
pub(super) fn emit_rich(env: &mut Env<'_>, targets: &[&CommandSenderInst<'_>], line: &str) {
    let mut api = Api::new(env);
    for target in targets {
        if let Err(e) = target.send_message(&mut api, line) {
            tracing::debug!("failed to send test output: {e}");
        }
    }
}

/// The owning plugin's name from `org.bukkit.plugin.Plugin#getName()`
fn plugin_name(env: &mut Env<'_>) -> eyre::Result<String> {
    let mut api = Api::new(env);
    let plugin = api.plugin()?;
    plugin.name(&mut api)
}
