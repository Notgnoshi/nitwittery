use jni::objects::{JString, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::jobject_repr::JObjectRepr;
use crate::{papermc_jobject, papermc_jobject_inst};

papermc_jobject_inst! {
    /// Mirrors `org.bukkit.command.CommandSender`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/command/CommandSender.html>.
    pub CommandSenderInst<'local> = "org/bukkit/command/CommandSender": CommandSender;
}

papermc_jobject! {
    /// Mirrors `org.bukkit.command.ConsoleCommandSender`.
    ///
    /// Obtain via [crate::bukkit::Bukkit::console_sender].
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/command/ConsoleCommandSender.html>.
    pub ConsoleCommandSender<'local> = "org/bukkit/command/ConsoleCommandSender": CommandSender;
}

/// Mirrors `org.bukkit.command.CommandSender`.
pub trait CommandSender<'local>: JObjectRepr<'local> {
    /// Mirrors `org.bukkit.command.CommandSender#getName()`.
    fn name(&self, api: &mut Api) -> eyre::Result<String> {
        let env = api.jni();
        let name_obj = env
            .call_method(
                self.as_jobject(),
                jni_str!("getName"),
                jni_sig!("()Ljava/lang/String;"),
                &[],
            )?
            .l()?;
        let name_jstr = env.cast_local::<JString>(name_obj)?;
        Ok(name_jstr.try_to_string(env)?)
    }

    /// Mirrors `net.kyori.adventure.audience.Audience#sendMessage(Component)`, parsing `msg` as
    /// MiniMessage (<https://docs.advntr.dev/minimessage/index.html>) first.
    ///
    /// For literal text, use [Self::send_plain].
    fn send_message(&self, api: &mut Api, msg: impl AsRef<str>) -> eyre::Result<()> {
        let env = api.jni();
        let component = super::mini_message::deserialize(env, msg.as_ref())?;
        env.call_method(
            self.as_jobject(),
            jni_str!("sendMessage"),
            jni_sig!("(Lnet/kyori/adventure/text/Component;)V"),
            &[JValue::Object(&component)],
        )?;
        Ok(())
    }

    /// Mirrors `net.kyori.adventure.audience.Audience#sendMessage(Component)`, wrapping `msg`
    /// as a literal `Component.text(String)` rather than parsing MiniMessage tags.
    fn send_plain(&self, api: &mut Api, msg: impl AsRef<str>) -> eyre::Result<()> {
        let env = api.jni();
        let jstr = env.new_string(msg.as_ref())?;
        let component = env
            .call_static_method(
                jni_str!("net/kyori/adventure/text/Component"),
                jni_str!("text"),
                jni_sig!("(Ljava/lang/String;)Lnet/kyori/adventure/text/TextComponent;"),
                &[JValue::Object(&jstr)],
            )?
            .l()?;
        env.call_method(
            self.as_jobject(),
            jni_str!("sendMessage"),
            jni_sig!("(Lnet/kyori/adventure/text/Component;)V"),
            &[JValue::Object(&component)],
        )?;
        Ok(())
    }
}
