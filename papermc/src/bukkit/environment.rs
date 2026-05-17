use crate::papermc_enum;

papermc_enum! {
    pub Environment in "org/bukkit/World$Environment" {
        Normal => "NORMAL",
        Nether => "NETHER",
        TheEnd => "THE_END",
        Custom => "CUSTOM",
    }
}
