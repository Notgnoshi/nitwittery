use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::{CommandMap, PluginManager};
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.Server`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/Server.html>.
    pub Server<'local> = "org/bukkit/Server";
}

impl<'local> Server<'local> {
    /// Mirrors `org.bukkit.Server#getPluginManager()`.
    pub fn plugin_manager(&self, api: &mut Api<'_, 'local>) -> eyre::Result<PluginManager<'local>> {
        let obj = api
            .jni()
            .call_method(
                &self.obj,
                jni_str!("getPluginManager"),
                jni_sig!("()Lorg/bukkit/plugin/PluginManager;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { PluginManager::from_jobject(obj) })
    }

    /// Mirrors `org.bukkit.Server#getCommandMap()` (a Paper API extension).
    pub fn command_map(&self, api: &mut Api<'_, 'local>) -> eyre::Result<CommandMap<'local>> {
        let obj = api
            .jni()
            .call_method(
                &self.obj,
                jni_str!("getCommandMap"),
                jni_sig!("()Lorg/bukkit/command/CommandMap;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { CommandMap::from_jobject(obj) })
    }
}

/// Mirrors the static facade `org.bukkit.Bukkit`.
///
/// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/Bukkit.html>.
pub struct Bukkit;

impl Bukkit {
    /// Mirrors `org.bukkit.Bukkit#getServer()`.
    pub fn server<'local>(api: &mut Api<'_, 'local>) -> eyre::Result<Server<'local>> {
        let obj = api
            .jni()
            .call_static_method(
                jni_str!("org/bukkit/Bukkit"),
                jni_str!("getServer"),
                jni_sig!("()Lorg/bukkit/Server;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { Server::from_jobject(obj) })
    }
}
