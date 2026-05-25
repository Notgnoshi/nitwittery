use jni::sys::{JNIEnv, jobject};
use papermc::{FnTable, Plugin, SetupApi};

use crate::config::NitwitteryConfig;

mod config;
mod features;

pub struct NitwitteryPlugin;

impl Plugin for NitwitteryPlugin {
    fn on_enable(api: &mut SetupApi<'_, '_, Self>) -> eyre::Result<Self> {
        let config = NitwitteryConfig::default();
        if config.initial_player_spawning.enabled {
            features::initial_player_spawning::enable(api, &config.initial_player_spawning)?;
        }
        Ok(NitwitteryPlugin)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn papermc_plugin_init(env: *mut JNIEnv, plugin: jobject) -> *const FnTable {
    papermc::init::<NitwitteryPlugin>(env, plugin)
}
