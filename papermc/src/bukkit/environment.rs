use crate::papermc_enum;

papermc_enum! {
    /// Mirrors `org.bukkit.World.Environment`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/World.Environment.html>.
    pub Environment in "org/bukkit/World$Environment" {
        Normal => "NORMAL",
        Nether => "NETHER",
        TheEnd => "THE_END",
        Custom => "CUSTOM",
    }
}
