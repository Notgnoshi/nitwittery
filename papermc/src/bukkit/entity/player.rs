use jni::objects::JValue;
use jni::{jni_sig, jni_str};

use super::Entity;
use crate::api::Api;
use crate::bukkit::{Audience, CommandSender, Location};
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.entity.Player`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/entity/Player.html>.
    pub Player<'local> = "org/bukkit/entity/Player": Entity, CommandSender, Audience;
}

impl<'local> Player<'local> {
    /// Mirrors `org.bukkit.entity.Entity#getLocation()`.
    pub fn location(&self, api: &mut Api<'_, 'local>) -> eyre::Result<Location<'local>> {
        let env = api.jni();
        let obj = env
            .call_method(
                &self.obj,
                jni_str!("getLocation"),
                jni_sig!("()Lorg/bukkit/Location;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { Location::from_jobject(obj) })
    }

    /// Mirrors `Player#getRespawnLocation()`.
    ///
    /// Returns `None` when the Java method returns null (the player has no bed, no charged
    /// anchor, and no plugin-set spawn).
    pub fn respawn_location(
        &self,
        api: &mut Api<'_, 'local>,
    ) -> eyre::Result<Option<Location<'local>>> {
        let env = api.jni();
        let obj = env
            .call_method(
                &self.obj,
                jni_str!("getRespawnLocation"),
                jni_sig!("()Lorg/bukkit/Location;"),
                &[],
            )?
            .l()?;
        if obj.is_null() {
            return Ok(None);
        }
        Ok(Some(unsafe { Location::from_jobject(obj) }))
    }

    /// Mirrors `Player#setRespawnLocation(Location, boolean)`.
    ///
    /// With `force=true`, the location is saved even if there is no bed or charged anchor at
    /// the target.
    pub fn set_respawn_location(
        &self,
        api: &mut Api<'_, 'local>,
        loc: &Location<'local>,
        force: bool,
    ) -> eyre::Result<()> {
        let env = api.jni();
        env.call_method(
            &self.obj,
            jni_str!("setRespawnLocation"),
            jni_sig!("(Lorg/bukkit/Location;Z)V"),
            &[JValue::Object(loc.as_jobject()), JValue::Bool(force)],
        )?;
        Ok(())
    }
}
