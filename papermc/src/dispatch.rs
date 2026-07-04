use std::sync::Arc;

use jni::objects::{JObject, JObjectArray, JString, JValue};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jlong, jobject, jobjectArray};
use jni::{Env, jni_sig, jni_str};
use tracing::warn;

use crate::{ctx, ffi};

pub(crate) type EventHandler = Arc<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>) + Send + Sync>;
pub(crate) type CommandHandler =
    Arc<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>, &[String]) -> bool + Send + Sync>;
/// Command tab-completion callback: receives the sender and the current args (the last element
/// is the partial word being completed). `None` falls back to Bukkit's default completion.
pub(crate) type TabCompleter =
    Arc<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>, &[String]) -> Option<Vec<String>> + Send + Sync>;

pub(crate) unsafe extern "C" fn dispatch_event(
    env: *mut jni::sys::JNIEnv,
    handler_id: jlong,
    event: jobject,
) {
    let _ = ffi::bridge(env, |env: &mut Env<'_>| -> eyre::Result<()> {
        let event_obj = unsafe { JObject::from_raw(env, event) };
        let handler = ctx::with_ctx(|c| c.event_handlers.get(&handler_id).cloned()).flatten();
        let Some(handler) = handler else {
            warn!("no event handler registered for id {handler_id}");
            return Ok(());
        };
        handler(env, &event_obj);
        Ok(())
    });
}

pub(crate) unsafe extern "C" fn dispatch_command(
    env: *mut jni::sys::JNIEnv,
    handler_id: jlong,
    sender: jobject,
    args: jobjectArray,
) -> jboolean {
    let result = ffi::bridge(env, |env: &mut Env<'_>| -> eyre::Result<bool> {
        let sender_obj = unsafe { JObject::from_raw(env, sender) };
        let args_arr = unsafe { JObjectArray::<JString>::from_raw(env, args) };
        let args_vec = read_string_array(env, &args_arr)?;
        let handler = ctx::with_ctx(|c| c.command_handlers.get(&handler_id).cloned()).flatten();
        let Some(handler) = handler else {
            warn!("no command handler registered for id {handler_id}");
            return Ok(false);
        };
        Ok(handler(env, &sender_obj, &args_vec))
    });
    match result {
        Ok(true) => JNI_TRUE,
        Ok(false) => JNI_FALSE,
        Err(_) => JNI_FALSE,
    }
}

pub(crate) unsafe extern "C" fn dispatch_tab_complete(
    env: *mut jni::sys::JNIEnv,
    handler_id: jlong,
    sender: jobject,
    args: jobjectArray,
) -> jobject {
    let result = ffi::bridge(env, |env: &mut Env<'_>| -> eyre::Result<jobject> {
        let sender_obj = unsafe { JObject::from_raw(env, sender) };
        let args_arr = unsafe { JObjectArray::<JString>::from_raw(env, args) };
        let args_vec = read_string_array(env, &args_arr)?;
        let completer = ctx::with_ctx(|c| c.tab_completers.get(&handler_id).cloned()).flatten();
        let Some(completer) = completer else {
            return Ok(std::ptr::null_mut());
        };
        let Some(completions) = completer(env, &sender_obj, &args_vec) else {
            return Ok(std::ptr::null_mut());
        };
        // Mirrors `java.util.ArrayList(int)` / `java.util.List#add(Object)`. The sized inner
        // frame bounds the per-completion string locals; the list itself is returned into the
        // caller's frame.
        let list = env.with_local_frame_returning_local::<_, JObject, eyre::Report>(
            completions.len() + 4,
            |env| -> eyre::Result<JObject<'_>> {
                let list = env.new_object(
                    jni_str!("java/util/ArrayList"),
                    jni_sig!("(I)V"),
                    &[JValue::Int(
                        i32::try_from(completions.len()).unwrap_or(i32::MAX),
                    )],
                )?;
                for completion in &completions {
                    let jstr = env.new_string(completion)?;
                    env.call_method(
                        &list,
                        jni_str!("add"),
                        jni_sig!("(Ljava/lang/Object;)Z"),
                        &[JValue::Object(&jstr)],
                    )?;
                }
                Ok(list)
            },
        )?;
        Ok(list.into_raw())
    });
    result.unwrap_or(std::ptr::null_mut())
}

fn read_string_array(
    env: &mut Env<'_>,
    arr: &JObjectArray<'_, JString>,
) -> jni::errors::Result<Vec<String>> {
    let len = arr.len(env)?;
    // Each `get_element` allocates a local JNI ref. JNI guarantees only 16 locals by default, so a
    // long argument list overflows the outer frame's allotment. Push a sized sub-frame so those
    // intermediates are released en masse when this helper returns
    env.with_local_frame(len + 4, |env| -> jni::errors::Result<Vec<String>> {
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            let elem = arr.get_element(env, i)?;
            let s = elem.try_to_string(env)?;
            out.push(s);
        }
        Ok(out)
    })
}
