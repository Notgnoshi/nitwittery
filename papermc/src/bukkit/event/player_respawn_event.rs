use jni::objects::JValue;
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::{Location, Player, RespawnReason};
use crate::papermc_event;

papermc_event! {
    /// Mirrors `org.bukkit.event.player.PlayerRespawnEvent`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/event/player/PlayerRespawnEvent.html>.
    ///
    /// Fires on the main server thread when a player respawns. The event's
    /// [PlayerRespawnEventRef::respawn_location] is the location vanilla has already resolved
    /// (bed, charged anchor, or world spawn); handlers can override it via
    /// [PlayerRespawnEventRef::set_respawn_location].
    pub PlayerRespawnEvent => PlayerRespawnEventRef
        = "org/bukkit/event/player/PlayerRespawnEvent";
}

impl<'local> PlayerRespawnEventRef<'local> {
    /// Mirrors `PlayerEvent#getPlayer()`.
    pub fn player(&self, api: &mut Api<'_, 'local>) -> eyre::Result<Player<'local>> {
        let env = api.jni();
        let obj = env
            .call_method(
                &self.obj,
                jni_str!("getPlayer"),
                jni_sig!("()Lorg/bukkit/entity/Player;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { Player::from_jobject(obj) })
    }

    /// Mirrors `PlayerRespawnEvent#getRespawnLocation()`.
    pub fn respawn_location(&self, api: &mut Api<'_, 'local>) -> eyre::Result<Location<'local>> {
        let env = api.jni();
        let obj = env
            .call_method(
                &self.obj,
                jni_str!("getRespawnLocation"),
                jni_sig!("()Lorg/bukkit/Location;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { Location::from_jobject(obj) })
    }

    /// Mirrors `PlayerRespawnEvent#setRespawnLocation(Location)`.
    pub fn set_respawn_location(
        &self,
        api: &mut Api<'_, 'local>,
        loc: &Location<'local>,
    ) -> eyre::Result<()> {
        let env = api.jni();
        env.call_method(
            &self.obj,
            jni_str!("setRespawnLocation"),
            jni_sig!("(Lorg/bukkit/Location;)V"),
            &[JValue::Object(loc.as_jobject())],
        )?;
        Ok(())
    }

    /// Mirrors `PlayerRespawnEvent#isBedSpawn()`.
    pub fn is_bed_spawn(&self, api: &mut Api<'_, 'local>) -> eyre::Result<bool> {
        let env = api.jni();
        Ok(env
            .call_method(&self.obj, jni_str!("isBedSpawn"), jni_sig!("()Z"), &[])?
            .z()?)
    }

    /// Mirrors `PlayerRespawnEvent#isAnchorSpawn()`.
    pub fn is_anchor_spawn(&self, api: &mut Api<'_, 'local>) -> eyre::Result<bool> {
        let env = api.jni();
        Ok(env
            .call_method(&self.obj, jni_str!("isAnchorSpawn"), jni_sig!("()Z"), &[])?
            .z()?)
    }

    /// Mirrors `PlayerRespawnEvent#isMissingRespawnBlock()`.
    ///
    /// True when vanilla tried to use a respawn block (bed or anchor) but it was missing -- i.e.
    /// the player's saved spawn block was destroyed and vanilla fell back to world spawn for
    /// this respawn. False both for normal bed/anchor respawns and for plugin-set non-block
    /// respawn locations.
    pub fn is_missing_respawn_block(&self, api: &mut Api<'_, 'local>) -> eyre::Result<bool> {
        let env = api.jni();
        Ok(env
            .call_method(
                &self.obj,
                jni_str!("isMissingRespawnBlock"),
                jni_sig!("()Z"),
                &[],
            )?
            .z()?)
    }

    /// Mirrors `PlayerRespawnEvent#getRespawnReason()`.
    pub fn respawn_reason(&self, api: &mut Api<'_, 'local>) -> eyre::Result<RespawnReason> {
        let reason_obj = {
            let env = api.jni();
            env.call_method(
                &self.obj,
                jni_str!("getRespawnReason"),
                jni_sig!("()Lorg/bukkit/event/player/PlayerRespawnEvent$RespawnReason;"),
                &[],
            )?
            .l()?
        };
        for variant in [
            RespawnReason::Death,
            RespawnReason::EndPortal,
            RespawnReason::Plugin,
        ] {
            let env = api.jni();
            let candidate = variant.as_java(env)?;
            if env.is_same_object(&reason_obj, &candidate)? {
                return Ok(variant);
            }
        }
        Err(eyre::eyre!(
            "unknown PlayerRespawnEvent.RespawnReason variant"
        ))
    }
}
