use jni::objects::JValue;
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::Command;
use crate::java::{Map, ToJava as _};
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.command.CommandMap`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/command/CommandMap.html>.
    pub CommandMap<'local> = "org/bukkit/command/CommandMap";
}

impl<'local> CommandMap<'local> {
    /// Mirrors `org.bukkit.command.CommandMap#register(String, Command)`.
    ///
    /// Returns false when the bare label conflicted with an existing registration; the command is
    /// then reachable only through its `fallback_prefix:label` alias.
    #[deprecated(note = "Prefer `SetupApi::register_command` instead")]
    pub fn register(
        &self,
        api: &mut Api<'_, 'local>,
        fallback_prefix: &str,
        command: &Command<'_>,
    ) -> eyre::Result<bool> {
        let prefix = fallback_prefix.to_java(api)?;
        Ok(api
            .jni()
            .call_method(
                &self.obj,
                jni_str!("register"),
                jni_sig!("(Ljava/lang/String;Lorg/bukkit/command/Command;)Z"),
                &[
                    JValue::Object(&prefix),
                    JValue::Object(command.as_jobject()),
                ],
            )?
            .z()?)
    }

    /// Mirrors `org.bukkit.command.CommandMap#getKnownCommands()` (a Paper API extension).
    ///
    /// The returned map is the live command registry (label -> command), not a snapshot; removing
    /// an entry is what actually unregisters a command, since `Command#unregister(CommandMap)` does
    /// not touch it.
    pub fn known_commands(&self, api: &mut Api<'_, 'local>) -> eyre::Result<Map<'local>> {
        let obj = api
            .jni()
            .call_method(
                &self.obj,
                jni_str!("getKnownCommands"),
                jni_sig!("()Ljava/util/Map;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { Map::from_jobject(obj) })
    }
}
