use crate::features;

/// Top-level plugin configuration.
///
/// Each feature owns its own `Config` struct under `features::<feature_name>::Config`. This module
/// aggregates them so future on-disk loading sees a single root value.
#[derive(Debug, Clone, Default)]
pub struct NitwitteryConfig {
    pub initial_player_spawning: features::initial_player_spawning::Config,
}
