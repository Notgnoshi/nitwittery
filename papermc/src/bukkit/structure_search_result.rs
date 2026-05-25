use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::Location;
use crate::papermc_jobject;

papermc_jobject! {
    /// Mirrors `org.bukkit.util.StructureSearchResult`.
    ///
    /// See <https://jd.papermc.io/paper/1.21.11/org/bukkit/util/StructureSearchResult.html>.
    pub StructureSearchResult<'local> = "org/bukkit/util/StructureSearchResult";
}

impl<'local> StructureSearchResult<'local> {
    /// Mirrors `StructureSearchResult#getLocation()`.
    pub fn location(&self, api: &mut Api<'_, 'local>) -> eyre::Result<Location<'local>> {
        let env = api.jni();
        let obj = env
            .call_method(
                &self.obj,
                jni_str!("getLocation"),
                jni_sig!("()Lorg/bukkit/Location;"),
                &[],
            )?
            .l()?;
        Ok(unsafe { Location::from_jobject(obj) })
    }
}
