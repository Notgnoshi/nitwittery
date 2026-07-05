use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.Server`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/Server.html>.
    pub Server<'local> = "org/bukkit/Server";
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
