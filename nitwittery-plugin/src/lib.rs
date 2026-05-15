use jni::sys::{JNIEnv, jobject};
use papermc::{FnTable, Plugin, SetupApi};

pub struct NitwitteryPlugin;

impl Plugin for NitwitteryPlugin {
    fn on_enable(_api: &mut SetupApi<'_, '_, Self>) -> eyre::Result<Self> {
        Ok(NitwitteryPlugin)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn papermc_plugin_init(env: *mut JNIEnv, plugin: jobject) -> *const FnTable {
    papermc::init::<NitwitteryPlugin>(env, plugin)
}
