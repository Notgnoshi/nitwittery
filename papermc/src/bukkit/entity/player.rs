use super::Entity;
use crate::bukkit::{Audience, CommandSender};
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.entity.Player`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/entity/Player.html>.
    pub Player<'local> = "org/bukkit/entity/Player": Entity, CommandSender, Audience;
}
