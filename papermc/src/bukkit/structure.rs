use crate::papermc_enum;

papermc_enum! {
    /// Mirrors `org.bukkit.generator.structure.Structure`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/generator/structure/Structure.html>.
    pub Structure in "org/bukkit/generator/structure/Structure" {
        AncientCity => "ANCIENT_CITY",
        BastionRemnant => "BASTION_REMNANT",
        BuriedTreasure => "BURIED_TREASURE",
        DesertPyramid => "DESERT_PYRAMID",
        EndCity => "END_CITY",
        Fortress => "FORTRESS",
        Igloo => "IGLOO",
        JunglePyramid => "JUNGLE_PYRAMID",
        Mansion => "MANSION",
        Mineshaft => "MINESHAFT",
        MineshaftMesa => "MINESHAFT_MESA",
        Monument => "MONUMENT",
        NetherFossil => "NETHER_FOSSIL",
        OceanRuinCold => "OCEAN_RUIN_COLD",
        OceanRuinWarm => "OCEAN_RUIN_WARM",
        PillagerOutpost => "PILLAGER_OUTPOST",
        RuinedPortal => "RUINED_PORTAL",
        RuinedPortalDesert => "RUINED_PORTAL_DESERT",
        RuinedPortalJungle => "RUINED_PORTAL_JUNGLE",
        RuinedPortalMountain => "RUINED_PORTAL_MOUNTAIN",
        RuinedPortalNether => "RUINED_PORTAL_NETHER",
        RuinedPortalOcean => "RUINED_PORTAL_OCEAN",
        RuinedPortalSwamp => "RUINED_PORTAL_SWAMP",
        Shipwreck => "SHIPWRECK",
        ShipwreckBeached => "SHIPWRECK_BEACHED",
        Stronghold => "STRONGHOLD",
        SwampHut => "SWAMP_HUT",
        TrailRuins => "TRAIL_RUINS",
        TrialChambers => "TRIAL_CHAMBERS",
        VillageDesert => "VILLAGE_DESERT",
        VillagePlains => "VILLAGE_PLAINS",
        VillageSavanna => "VILLAGE_SAVANNA",
        VillageSnowy => "VILLAGE_SNOWY",
        VillageTaiga => "VILLAGE_TAIGA",
    }
}

impl Structure {
    /// All five village biome variants, in a stable order.
    pub const VILLAGES: [Structure; 5] = [
        Structure::VillagePlains,
        Structure::VillageDesert,
        Structure::VillageSavanna,
        Structure::VillageSnowy,
        Structure::VillageTaiga,
    ];
}
