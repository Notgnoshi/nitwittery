use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::Player;
use crate::papermc_event;

papermc_event! {
    /// Mirrors `org.bukkit.event.player.PlayerJoinEvent`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/event/player/PlayerJoinEvent.html>.
    ///
    /// Fires on the main server thread after the `Player` entity exists in-world. Suitable for
    /// any entity-touching work that the async spawn-location event cannot do.
    pub PlayerJoinEvent => PlayerJoinEventRef
        = "org/bukkit/event/player/PlayerJoinEvent";
}

impl<'local> PlayerJoinEventRef<'local> {
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
