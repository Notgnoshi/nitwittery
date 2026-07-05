use crate::papermc_enum;

papermc_enum! {
    /// Mirrors `org.bukkit.event.EventPriority`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/event/EventPriority.html>.
    pub EventPriority in "org/bukkit/event/EventPriority" {
        Lowest => "LOWEST",
        Low => "LOW",
        Normal => "NORMAL",
        High => "HIGH",
        Highest => "HIGHEST",
        Monitor => "MONITOR",
    }
}
