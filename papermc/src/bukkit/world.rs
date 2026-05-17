use jni::objects::JValue;
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::{Environment, Location};
use crate::papermc_jobject;

papermc_jobject! {
    pub World<'local> = "org/bukkit/World";
}

impl<'local> World<'local> {
    pub fn environment(&self, api: &mut Api<'_, 'local>) -> eyre::Result<Environment> {
        let env_obj = {
            let env = api.jni();
            env.call_method(
                &self.obj,
                jni_str!("getEnvironment"),
                jni_sig!("()Lorg/bukkit/World$Environment;"),
                &[],
            )?
            .l()?
        };
        for variant in [
            Environment::Normal,
            Environment::Nether,
            Environment::TheEnd,
            Environment::Custom,
        ] {
            let env = api.jni();
            let candidate = variant.as_java(env)?;
            if env.is_same_object(&env_obj, &candidate)? {
                return Ok(variant);
            }
        }
        Err(eyre::eyre!("unknown World.Environment variant"))
    }

    pub fn spawn_location(&self, api: &mut Api<'_, 'local>) -> eyre::Result<Location<'local>> {
        let env = api.jni();
        let obj = env
            .call_method(
                &self.obj,
                jni_str!("getSpawnLocation"),
                jni_sig!("()Lorg/bukkit/Location;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { Location::from_jobject(obj) })
    }

    pub fn highest_block_y_at(
        &self,
        api: &mut Api<'_, 'local>,
        x: i32,
        z: i32,
    ) -> eyre::Result<i32> {
        let env = api.jni();
        Ok(env
            .call_method(
                &self.obj,
                jni_str!("getHighestBlockYAt"),
                jni_sig!("(II)I"),
                &[JValue::Int(x), JValue::Int(z)],
            )?
            .i()?)
    }
}
