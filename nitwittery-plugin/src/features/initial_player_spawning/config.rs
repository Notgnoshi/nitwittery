#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Config {
    /// Master switch for this feature. When false, all handlers are no-ops.
    pub enabled: bool,
    /// Radius (blocks) of the disc around vanilla spawn from which the random anchor is sampled.
    pub max_distance_from_spawn: f64,
    /// Maximum number of anchor-and-locate attempts before falling through to vanilla.
    pub max_attempts: u32,
    /// Per-attempt search radius (chunks) passed to `World.locateNearestStructure`.
    ///
    /// Bukkit measures locate radius in chunks, not blocks; 128 chunks is 2048 blocks.
    pub locate_radius_chunks: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            max_distance_from_spawn: 2000.0,
            max_attempts: 3,
            locate_radius_chunks: 128,
        }
    }
}
