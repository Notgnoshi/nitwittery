use jni::objects::JValue;
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::{Environment, Location, Structure, StructureSearchResult};
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.World`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/World.html>.
    pub World<'local> = "org/bukkit/World";
}

impl<'local> World<'local> {
    /// Get every world currently loaded on the server.
    ///
    /// Mirrors `org.bukkit.Bukkit#getWorlds()`
    pub fn all(api: &mut Api<'_, 'local>) -> eyre::Result<Vec<World<'local>>> {
        let env = api.jni();
        let list = env
            .call_static_method(
                jni_str!("org/bukkit/Bukkit"),
                jni_str!("getWorlds"),
                jni_sig!("()Ljava/util/List;"),
                &[],
            )?
            .l()?;
        let size = env
            .call_method(&list, jni_str!("size"), jni_sig!("()I"), &[])?
            .i()?;
        let mut worlds = Vec::with_capacity(size as usize);
        for i in 0..size {
            let obj = env
                .call_method(
                    &list,
                    jni_str!("get"),
                    jni_sig!("(I)Ljava/lang/Object;"),
                    &[JValue::Int(i)],
                )?
                .l()?;
            worlds.push(unsafe { World::from_jobject(obj) });
        }
        Ok(worlds)
    }

    /// Mirrors `org.bukkit.World#getEnvironment()`.
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

    /// Mirrors `org.bukkit.World#getSpawnLocation()`.
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

    /// Mirrors `org.bukkit.World#getHighestBlockYAt(int, int)`.
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

    /// Mirrors `World#locateNearestStructure(Location, Structure, int, boolean)`.
    ///
    /// With `find_unexplored=true`, the call may generate chunks and block the calling thread;
    /// must therefore be called from the main server thread.
    pub fn locate_nearest_structure(
        &self,
        api: &mut Api<'_, 'local>,
        origin: &Location<'local>,
        structure: Structure,
        radius: i32,
        find_unexplored: bool,
    ) -> eyre::Result<Option<StructureSearchResult<'local>>> {
        let structure_obj = {
            let env = api.jni();
            structure.as_java(env)?
        };
        let env = api.jni();
        let result = env
            .call_method(
                &self.obj,
                jni_str!("locateNearestStructure"),
                jni_sig!(
                    "(Lorg/bukkit/Location;Lorg/bukkit/generator/structure/Structure;IZ)\
                     Lorg/bukkit/util/StructureSearchResult;"
                ),
                &[
                    JValue::Object(origin.as_jobject()),
                    JValue::Object(&structure_obj),
                    JValue::Int(radius),
                    JValue::Bool(find_unexplored),
                ],
            )?
            .l()?;
        if result.is_null() {
            return Ok(None);
        }
        Ok(Some(unsafe { StructureSearchResult::from_jobject(result) }))
    }
}
