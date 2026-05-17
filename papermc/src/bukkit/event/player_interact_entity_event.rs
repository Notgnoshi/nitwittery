use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::EntityInst;
use crate::papermc_event;

papermc_event! {
    /// Mirrors `org.bukkit.event.player.PlayerInteractEntityEvent`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/event/player/PlayerInteractEntityEvent.html>.
    pub PlayerInteractEntityEvent => PlayerInteractEntityEventRef
        = "org/bukkit/event/player/PlayerInteractEntityEvent";
}

impl<'local> PlayerInteractEntityEventRef<'local> {
    /// Mirrors `org.bukkit.event.player.PlayerInteractEntityEvent#getRightClicked()`.
    pub fn right_clicked(&self, api: &mut Api<'_, 'local>) -> eyre::Result<EntityInst<'local>> {
        let env = api.jni();
        let entity = env
            .call_method(
                &self.obj,
                jni_str!("getRightClicked"),
                jni_sig!("()Lorg/bukkit/entity/Entity;"),
                &[],
            )?
            .l()?;
        Ok(EntityInst::new(entity))
    }
}
