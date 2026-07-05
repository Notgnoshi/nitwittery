use jni::objects::JValue;
use jni::{Env, jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::{Bukkit, Command, EventPriority, HandlerList};
use crate::ctx;
use crate::java::ToJava as _;
use crate::jobject_repr::JObjectRepr as _;

pub(crate) fn subscribe_event<'local>(
    env: &mut Env<'local>,
    event_class_name: &'static str,
    handler_id: i64,
) -> eyre::Result<()> {
    let mut api = Api::new(env);
    let event_class = api.class(event_class_name)?;
    let executor = api.jni().new_object(
        jni_str!("io/papermc/RustEventExecutor"),
        jni_sig!("(J)V"),
        &[JValue::Long(handler_id)],
    )?;
    let plugin = api.plugin()?;
    let server = plugin.server(&mut api)?;
    let plugin_manager = server.plugin_manager(&mut api)?;
    #[allow(deprecated)]
    plugin_manager.register_event(
        &mut api,
        &event_class,
        &executor,
        EventPriority::Normal,
        &executor,
        &plugin,
    )?;
    Ok(())
}

pub(crate) fn register_command<'local>(
    env: &mut Env<'local>,
    name: &str,
    permission: Option<&str>,
    completer: Option<crate::dispatch::TabCompleter>,
    handler_id: i64,
) -> eyre::Result<()> {
    if let Some(completer) = completer {
        ctx::with_ctx(|c| c.tab_completers.insert(handler_id, completer))
            .expect("Ctx installed during plugin_init");
    }
    let mut api = Api::new(env);
    let name_jstr = name.to_java(&mut api)?;
    let command_obj = api.jni().new_object(
        jni_str!("io/papermc/RustCommand"),
        jni_sig!("(Ljava/lang/String;J)V"),
        &[JValue::Object(&name_jstr), JValue::Long(handler_id)],
    )?;
    // RustCommand extends org.bukkit.command.Command.
    let command = unsafe { Command::from_jobject(command_obj) };
    if permission.is_some() {
        #[allow(deprecated)]
        command.set_permission(&mut api, permission)?;
    }
    let plugin = api.plugin()?;
    let plugin_name = plugin.name(&mut api)?;
    let fallback = plugin_name.trim().to_lowercase();
    let label = name.trim().to_lowercase();
    let command_map = plugin.server(&mut api)?.command_map(&mut api)?;
    #[allow(deprecated)]
    command_map.register(&mut api, &plugin_name, &command)?;
    let cmd_global = api.jni().new_global_ref(command.as_jobject())?;
    ctx::with_ctx(|c| {
        c.registered_commands.push(ctx::RegisteredCommand {
            command: cmd_global,
            label,
            fallback,
        })
    })
    .expect("Ctx installed during plugin_init");
    Ok(())
}

pub(crate) fn unregister_commands(env: &mut Env<'_>) -> eyre::Result<()> {
    let commands =
        ctx::with_ctx(|c| std::mem::take(&mut c.registered_commands)).unwrap_or_default();
    if commands.is_empty() {
        return Ok(());
    }
    let mut api = Api::new(env);
    let command_map = Bukkit::server(&mut api)?.command_map(&mut api)?;
    // `org.bukkit.command.Command#unregister(CommandMap)` only clears the command's own
    // back-reference; the map keeps its `knownCommands` entries, which would leave stale
    // commands dispatching dead handler ids after a `/reload`. Remove our entries from the
    // live map directly.
    let known = command_map.known_commands(&mut api)?;
    for cmd in commands {
        let qualified = format!("{}:{}", cmd.fallback, cmd.label);
        for key in [cmd.label.as_str(), qualified.as_str()] {
            let key_jstr = key.to_java(&mut api)?;
            // Only remove entries that still point at our command
            if let Some(current) = known.get(&mut api, &key_jstr)?
                && api.jni().is_same_object(&current, &*cmd.command)?
            {
                known.remove(&mut api, &key_jstr)?;
            }
        }
        let command_local = api.jni().new_local_ref(&*cmd.command)?;
        let command = unsafe { Command::from_jobject(command_local) };
        #[allow(deprecated)]
        let _ = command.unregister(&mut api, &command_map);
    }
    Ok(())
}

/// Must run before handler-map teardown; otherwise an event in flight between teardown and
/// Bukkit's own listener cleanup logs a spurious "no handler registered" warning.
pub(crate) fn unregister_all_listeners(env: &mut Env<'_>) -> eyre::Result<()> {
    let mut api = Api::new(env);
    let plugin = api.plugin()?;
    HandlerList::unregister_all(&mut api, &plugin)
}
