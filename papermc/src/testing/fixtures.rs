use crate::bukkit::{CommandSenderInst, Player, World};
use crate::jobject_repr::{JClassCast, JObjectRepr};
use crate::testing::{Fixture, TestCtx, TestFixture};

/// The server's default world: the first entry of `org.bukkit.Bukkit#getWorlds()`.
impl<'a, 'l> TestFixture<'a, 'l> for World<'l> {
    fn extract(ctx: &mut TestCtx<'a, 'l>) -> eyre::Result<Fixture<Self>> {
        let mut worlds = World::all(&mut ctx.api)?;
        if worlds.is_empty() {
            // A running server always has at least one world
            return Err(eyre::eyre!("Bukkit.getWorlds() returned an empty list"));
        }
        Ok(Fixture::Present(worlds.swap_remove(0)))
    }
}

/// The `/test` invoker as a plain command sender; always present.
impl<'a, 'l> TestFixture<'a, 'l> for CommandSenderInst<'l> {
    fn extract(ctx: &mut TestCtx<'a, 'l>) -> eyre::Result<Fixture<Self>> {
        let obj = ctx.api.jni().new_local_ref(ctx.invoker.as_jobject())?;
        Ok(Fixture::Present(CommandSenderInst::new(obj)))
    }
}

/// The invoker as a [Player]; skipped when `/test` runs from the console.
impl<'a, 'l> TestFixture<'a, 'l> for Player<'l> {
    fn extract(ctx: &mut TestCtx<'a, 'l>) -> eyre::Result<Fixture<Self>> {
        let class = ctx.api.class(Player::CLASS_NAME)?;
        let is_player = {
            let env = ctx.api.jni();
            env.is_instance_of(ctx.invoker.as_jobject(), &class)?
        };
        if !is_player {
            return Ok(Fixture::Skip("needs player"));
        }
        let obj = ctx.api.jni().new_local_ref(ctx.invoker.as_jobject())?;
        // SAFETY: just verified the invoker is an org.bukkit.entity.Player instance.
        Ok(Fixture::Present(unsafe { Player::from_jobject(obj) }))
    }
}
