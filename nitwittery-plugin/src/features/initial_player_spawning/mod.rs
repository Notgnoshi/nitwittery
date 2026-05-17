mod config;
mod handlers;
mod locate;

pub use config::Config;
use papermc::SetupApi;

use crate::NitwitteryPlugin;

/// Register the initial_player_spawning feature's event handlers.
///
/// Called from `Plugin::on_enable`
pub fn enable(_api: &mut SetupApi<'_, '_, NitwitteryPlugin>, _config: &Config) -> eyre::Result<()> {
    Ok(())
}
