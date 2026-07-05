use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::CommandMap;
use crate::java::ToJava as _;
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.command.Command`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/command/Command.html>.
    pub Command<'local> = "org/bukkit/command/Command";
}

impl<'local> Command<'local> {
    /// Mirrors `org.bukkit.command.Command#setPermission(String)`.
    ///
    /// Sets the node the command checks; it does not declare the node with the PluginManager,
    /// and undeclared nodes are treated as op-only.
    #[deprecated(note = "Prefer the `permission` parameter of `SetupApi::register_command`")]
    pub fn set_permission(
        &self,
        api: &mut Api<'_, 'local>,
        permission: Option<&str>,
    ) -> eyre::Result<()> {
        let jstr = match permission {
            Some(permission) => JObject::from(permission.to_java(api)?),
            None => JObject::null(),
        };
        api.jni().call_method(
            &self.obj,
            jni_str!("setPermission"),
            jni_sig!("(Ljava/lang/String;)V"),
            &[JValue::Object(&jstr)],
        )?;
        Ok(())
    }

    /// Mirrors `org.bukkit.command.Command#unregister(CommandMap)`.
    ///
    /// This only clears the command's own back-reference: the map keeps its `knownCommands`
    /// entries, so the command still dispatches until those are removed through
    /// [CommandMap::known_commands].
    #[deprecated(
        note = "Prefer `SetupApi::register_command`, which handles unregistration on teardown automatically"
    )]
    pub fn unregister(
        &self,
        api: &mut Api<'_, 'local>,
        command_map: &CommandMap<'_>,
    ) -> eyre::Result<bool> {
        Ok(api
            .jni()
            .call_method(
                &self.obj,
                jni_str!("unregister"),
                jni_sig!("(Lorg/bukkit/command/CommandMap;)Z"),
                &[JValue::Object(command_map.as_jobject())],
            )?
            .z()?)
    }
}
