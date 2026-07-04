use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::Instant;

use jni::objects::{JObject, JString};
use jni::{Env, jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::{CommandSender as _, CommandSenderInst};
use crate::jobject_repr::JClassCast as _;
use crate::testing::{TESTS, TestCase, TestCtx, TestOutcome, args};
use crate::{ctx, registration};

/// Register the `/test` command.
///
/// The handler runs the whole battery inline on the main thread, blocking the current tick until
/// it finishes.
pub(crate) fn register_test_command(env: &mut Env<'_>) -> eyre::Result<()> {
    let id = ctx::next_id();
    ctx::with_ctx(|c| {
        c.command_handlers.insert(
            id,
            Arc::new(|env, sender_obj, args| {
                if let Err(e) = run_battery(env, sender_obj, args) {
                    tracing::error!("/test battery failed to run: {e:?}");
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

fn run_battery(env: &mut Env<'_>, sender_obj: &JObject<'_>, args: &[String]) -> eyre::Result<()> {
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

    let mut cases: Vec<&TestCase> = TESTS
        .iter()
        .filter(|c| args::matches(&spec, c.name))
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

    let start = Instant::now();
    let plural = if cases.len() == 1 { "" } else { "s" };
    emit(
        env,
        sender,
        &format!("[{plugin}] running {} test{plural}", cases.len()),
    );

    let mut passed = 0usize;
    let mut ignored = 0usize;
    let mut skipped = 0usize;
    let mut failures: Vec<(&'static str, String)> = Vec::new();
    for case in cases {
        if case.ignored {
            ignored += 1;
            match case.ignore_reason {
                Some(reason) => {
                    emit(
                        env,
                        sender,
                        &format!("[{plugin}] test {} ... ignored, {reason}", case.name),
                    );
                }
                None => emit(
                    env,
                    sender,
                    &format!("[{plugin}] test {} ... ignored", case.name),
                ),
            }
            continue;
        }
        tracing::info!("test {} starting", case.name);
        match run_one(env, case) {
            TestOutcome::Passed => {
                passed += 1;
                emit(
                    env,
                    sender,
                    &format!("[{plugin}] test {} ... ok", case.name),
                );
            }
            TestOutcome::Failed(message) => {
                emit(
                    env,
                    sender,
                    &format!("[{plugin}] test {} ... FAILED", case.name),
                );
                failures.push((case.name, message));
            }
            TestOutcome::Skipped(reason) => {
                skipped += 1;
                emit(
                    env,
                    sender,
                    &format!("[{plugin}] test {} ... skipped ({reason})", case.name),
                );
            }
        }
    }

    if !failures.is_empty() {
        emit(env, sender, &format!("[{plugin}] failures:"));
        for (name, message) in &failures {
            emit(env, sender, &format!("[{plugin}] ---- {name} ----"));
            for line in message.lines() {
                emit(env, sender, &format!("[{plugin}] {line}"));
            }
        }
    }

    // A battery that found no tests is a failure: with filters it means a typo'd filter; without
    // filters it almost certainly means registry entries were stripped at link time rather than
    // that nobody wrote tests.
    if passed + ignored + skipped == 0 && failures.is_empty() && !spec.filters.is_empty() {
        emit(
            env,
            sender,
            &format!("[{plugin}] no tests matched the given filters"),
        );
    }
    let ok = failures.is_empty() && passed + ignored + skipped > 0;
    let verdict = if ok { "ok" } else { "FAILED" };
    let elapsed = start.elapsed().as_secs_f64();
    emit(
        env,
        sender,
        &format!(
            "[{plugin}] test result: {verdict}. {passed} passed; {} failed; {ignored} ignored; \
             {skipped} skipped; finished in {elapsed:.2}s",
            failures.len(),
        ),
    );
    Ok(())
}

/// Run one test inside its own JNI local frame so its local refs are bulk-freed on exit and
/// cannot dangle into the next test. A panicking test is contained and reported as failed.
fn run_one(env: &mut Env<'_>, case: &TestCase) -> TestOutcome {
    let result = env.with_local_frame(32, |env| -> jni::errors::Result<TestOutcome> {
        let mut tctx = TestCtx { api: Api::new(env) };
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
fn emit(env: &mut Env<'_>, sender: &CommandSenderInst<'_>, line: &str) {
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
