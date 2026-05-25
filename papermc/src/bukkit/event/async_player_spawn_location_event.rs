use jni::objects::JValue;
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::Location;
use crate::papermc_event;

papermc_event! {
    /// Mirrors `io.papermc.paper.event.player.AsyncPlayerSpawnLocationEvent`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/io/papermc/paper/event/player/AsyncPlayerSpawnLocationEvent.html>.
    ///
    /// Fires during the player configuration phase, off the main server thread, before the
    /// `Player` entity exists in-world. Handlers must not touch the player entity; only the
    /// event's spawn-location getter/setter and the connection profile (UUID, name) are safe.
    /// For main-thread work, dispatch via [crate::Api::run_sync].
    pub AsyncPlayerSpawnLocationEvent => AsyncPlayerSpawnLocationEventRef
        = "io/papermc/paper/event/player/AsyncPlayerSpawnLocationEvent";
}

impl<'local> AsyncPlayerSpawnLocationEventRef<'local> {
    /// Mirrors `AsyncPlayerSpawnLocationEvent#isNewPlayer()`.
    pub fn is_new_player(&self, api: &mut Api<'_, 'local>) -> eyre::Result<bool> {
        let env = api.jni();
        Ok(env
            .call_method(&self.obj, jni_str!("isNewPlayer"), jni_sig!("()Z"), &[])?
            .z()?)
    }

    /// Mirrors `AsyncPlayerSpawnLocationEvent#getSpawnLocation()`.
    pub fn spawn_location(&self, api: &mut Api<'_, 'local>) -> eyre::Result<Location<'local>> {
        let env = api.jni();
        let obj = env
            .call_method(
                &self.obj,
                jni_str!("getSpawnLocation"),
                jni_sig!("()Lorg/bukkit/Location;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { Location::from_jobject(obj) })
    }

    /// Mirrors `AsyncPlayerSpawnLocationEvent#setSpawnLocation(Location)`.
    pub fn set_spawn_location(
        &self,
        api: &mut Api<'_, 'local>,
        loc: &Location<'local>,
    ) -> eyre::Result<()> {
        let env = api.jni();
        env.call_method(
            &self.obj,
            jni_str!("setSpawnLocation"),
            jni_sig!("(Lorg/bukkit/Location;)V"),
            &[JValue::Object(loc.as_jobject())],
        )?;
        Ok(())
    }
}
