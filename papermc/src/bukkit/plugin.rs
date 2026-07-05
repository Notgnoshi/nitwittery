use jni::objects::JString;
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::Server;
use crate::java::FromJava as _;
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.plugin.Plugin`.
    ///
    /// Obtain the owning plugin's handle via [Api::plugin].
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/plugin/Plugin.html>.
    pub Plugin<'local> = "org/bukkit/plugin/Plugin";
}

impl<'local> Plugin<'local> {
    /// Mirrors `org.bukkit.plugin.Plugin#getName()`.
    pub fn name(&self, api: &mut Api<'_, 'local>) -> eyre::Result<String> {
        let name_obj = api
            .jni()
            .call_method(
                &self.obj,
                jni_str!("getName"),
                jni_sig!("()Ljava/lang/String;"),
                &[],
            )?
            .l()?;
        let name_jstr = api.jni().cast_local::<JString>(name_obj)?;
        String::from_java(api, &name_jstr)
    }

    /// Mirrors `org.bukkit.plugin.Plugin#getServer()`.
    pub fn server(&self, api: &mut Api<'_, 'local>) -> eyre::Result<Server<'local>> {
        let obj = api
            .jni()
            .call_method(
                &self.obj,
                jni_str!("getServer"),
                jni_sig!("()Lorg/bukkit/Server;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { Server::from_jobject(obj) })
    }
}
