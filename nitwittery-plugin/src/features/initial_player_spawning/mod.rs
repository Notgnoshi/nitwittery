mod config;
mod handlers;
pub(crate) mod locate;

pub use config::Config;
use papermc::SetupApi;
use papermc::bukkit::event::{
    AsyncPlayerSpawnLocationEvent, PlayerJoinEvent, PlayerPostRespawnEvent, PlayerRespawnEvent,
};

use crate::NitwitteryPlugin;

/// Register the initial_player_spawning feature's event handlers.
///
/// Called from `Plugin::on_enable`. The orchestrator is responsible for the `config.enabled`
/// gate; this function unconditionally registers handlers when called.
pub fn enable(api: &mut SetupApi<'_, '_, NitwitteryPlugin>, config: &Config) -> eyre::Result<()> {
    let cfg = *config;
    api.register_event::<AsyncPlayerSpawnLocationEvent, _>(move |plugin, api, event| {
        handlers::handle_async_spawn_location(plugin, api, event, cfg)
    })?;
    api.register_event::<PlayerJoinEvent, _>(move |plugin, api, event| {
        handlers::handle_player_join(plugin, api, event, cfg)
    })?;
    api.register_event::<PlayerRespawnEvent, _>(move |plugin, api, event| {
        handlers::handle_player_respawn(plugin, api, event, cfg)
    })?;
    api.register_event::<PlayerPostRespawnEvent, _>(move |plugin, api, event| {
        handlers::handle_player_post_respawn(plugin, api, event, cfg)
    })?;
    Ok(())
}
