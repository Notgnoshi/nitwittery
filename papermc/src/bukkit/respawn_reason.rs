use crate::papermc_enum;

papermc_enum! {
    /// Mirrors `org.bukkit.event.player.PlayerRespawnEvent.RespawnReason`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/event/player/PlayerRespawnEvent.RespawnReason.html>.
    ///
    /// Anchor respawns are reported as [RespawnReason::Death] flagged via
    /// `PlayerRespawnEvent#isAnchorSpawn()` rather than a dedicated variant.
    pub RespawnReason in "org/bukkit/event/player/PlayerRespawnEvent$RespawnReason" {
        Death => "DEATH",
        EndPortal => "END_PORTAL",
        Plugin => "PLUGIN",
    }
}
