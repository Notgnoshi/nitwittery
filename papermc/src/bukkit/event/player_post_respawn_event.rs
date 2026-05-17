use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::Player;
use crate::papermc_event;

papermc_event! {
    /// Mirrors `com.destroystokyo.paper.event.player.PlayerPostRespawnEvent`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/com/destroystokyo/paper/event/player/PlayerPostRespawnEvent.html>.
    ///
    /// Fires on the main server thread after the player has been teleported to the destination
    /// of a respawn. Suitable for persisting state that would be clobbered if set during
    /// `PlayerRespawnEvent` itself (e.g. calls to `Player#setRespawnLocation` made mid-respawn
    /// do not stick).
    pub PlayerPostRespawnEvent => PlayerPostRespawnEventRef
        = "com/destroystokyo/paper/event/player/PlayerPostRespawnEvent";
}

impl<'local> PlayerPostRespawnEventRef<'local> {
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
}
