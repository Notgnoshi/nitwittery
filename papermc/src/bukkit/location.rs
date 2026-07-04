use jni::objects::JValue;
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::World;
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.Location`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/Location.html>.
    pub Location<'local> = "org/bukkit/Location";
}

impl<'local> Location<'local> {
    /// Mirrors `org.bukkit.Location(World, double, double, double)`.
    pub fn new(
        api: &mut Api<'_, 'local>,
        world: &World<'_>,
        x: f64,
        y: f64,
        z: f64,
    ) -> eyre::Result<Self> {
        let class = api.class("org/bukkit/Location")?;
        let env = api.jni();
        let obj = env.new_object(
            &class,
            jni_sig!("(Lorg/bukkit/World;DDD)V"),
            &[
                JValue::Object(world.as_jobject()),
                JValue::Double(x),
                JValue::Double(y),
                JValue::Double(z),
            ],
        )?;
        Ok(Self { obj })
    }

    /// Mirrors `org.bukkit.Location#getWorld()`.
    ///
    /// The Java return is `@Nullable`; returns `None` for a world-less Location.
    pub fn world(&self, api: &mut Api<'_, 'local>) -> eyre::Result<Option<World<'local>>> {
        let env = api.jni();
        let obj = env
            .call_method(
                &self.obj,
                jni_str!("getWorld"),
                jni_sig!("()Lorg/bukkit/World;"),
                &[],
            )?
            .l()?;
        if obj.is_null() {
            return Ok(None);
        }
        Ok(Some(unsafe { World::from_jobject(obj) }))
    }

    /// Mirrors `org.bukkit.Location#getX()`.
    pub fn x(&self, api: &mut Api<'_, 'local>) -> eyre::Result<f64> {
        let env = api.jni();
        Ok(env
            .call_method(&self.obj, jni_str!("getX"), jni_sig!("()D"), &[])?
            .d()?)
    }

    /// Mirrors `org.bukkit.Location#getY()`.
    pub fn y(&self, api: &mut Api<'_, 'local>) -> eyre::Result<f64> {
        let env = api.jni();
        Ok(env
            .call_method(&self.obj, jni_str!("getY"), jni_sig!("()D"), &[])?
            .d()?)
    }

    /// Mirrors `org.bukkit.Location#getZ()`.
    pub fn z(&self, api: &mut Api<'_, 'local>) -> eyre::Result<f64> {
        let env = api.jni();
        Ok(env
            .call_method(&self.obj, jni_str!("getZ"), jni_sig!("()D"), &[])?
            .d()?)
    }

    /// Mirrors `org.bukkit.Location#getBlockX()`.
    pub fn block_x(&self, api: &mut Api<'_, 'local>) -> eyre::Result<i32> {
        let env = api.jni();
        Ok(env
            .call_method(&self.obj, jni_str!("getBlockX"), jni_sig!("()I"), &[])?
            .i()?)
    }

    /// Mirrors `org.bukkit.Location#getBlockZ()`.
    pub fn block_z(&self, api: &mut Api<'_, 'local>) -> eyre::Result<i32> {
        let env = api.jni();
        Ok(env
            .call_method(&self.obj, jni_str!("getBlockZ"), jni_sig!("()I"), &[])?
            .i()?)
    }
}
