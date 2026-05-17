use crate::jobject_repr::JObjectRepr;
use crate::papermc_jobject_inst;

mod player;
mod sheep;

pub use player::Player;
pub use sheep::Sheep;

papermc_jobject_inst! {
    /// Mirrors `org.bukkit.entity.Entity`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/entity/Entity.html>.
    pub EntityInst<'local> = "org/bukkit/entity/Entity": Entity;
}

/// Mirrors `org.bukkit.entity.Entity`.
pub trait Entity<'local>: JObjectRepr<'local> {}
