use jni::objects::JObject;
use jni::refs::Global;
use papermc::Api;
use papermc::bukkit::event::{
    AsyncPlayerSpawnLocationEventRef, PlayerJoinEventRef, PlayerPostRespawnEventRef,
    PlayerRespawnEventRef,
};
use papermc::bukkit::{Environment, Location, RespawnReason};
use papermc::jobject_repr::JObjectRepr;

use super::Config;
use super::locate::{find_village_coords_inline, find_village_location};
use crate::NitwitteryPlugin;

/// Bias the first-join spawn location toward a nearby village.
///
/// Runs off the main server thread; village-locate work is dispatched to the main thread via
/// [Api::run_sync]. Setting the event's spawn location itself is safe from the async context.
pub(super) fn handle_async_spawn_location<'local>(
    _plugin: &mut NitwitteryPlugin,
    api: &mut Api<'_, 'local>,
    event: &AsyncPlayerSpawnLocationEventRef<'local>,
    config: Config,
) -> eyre::Result<()> {
    if !event.is_new_player(api)? {
        return Ok(());
    }

    let spawn = event.spawn_location(api)?;
    let Some(world) = spawn.world(api)? else {
        tracing::debug!("async spawn: spawn location has no world; skipping");
        return Ok(());
    };
    if world.environment(api)? != Environment::Normal {
        return Ok(());
    }

    let cx = spawn.x(api)?;
    let cz = spawn.z(api)?;

    // Promote the world JObject to a Global so it can cross the netty -> main thread boundary
    // inside the run_sync closure.
    let world_global: Global<JObject<'static>> = api.jni().new_global_ref(world.as_jobject())?;

    let coords = api
        .run_sync(move |sync_api| find_village_location(sync_api, &world_global, cx, cz, config))?;

    let Some((vx, vy, vz)) = coords else {
        tracing::debug!("async spawn: no village within reach; falling through to vanilla");
        return Ok(());
    };

    let target = Location::new(api, &world, vx, vy, vz)?;
    event.set_spawn_location(api, &target)?;
    tracing::info!(x = vx, y = vy, z = vz, "biased first-join spawn to village");
    Ok(())
}

/// Persist the player's current location as their respawn point when they have none.
///
/// Same logic shared by [handle_player_join] and [handle_player_post_respawn]: if
/// `Player#getRespawnLocation()` is null, force-set it to the player's current location. Pinning
/// happens after the spawn/respawn flow completes, never during it, because calls to
/// `Player#setRespawnLocation` made during `PlayerRespawnEvent` are clobbered by Paper's
/// respawn-completion logic.
fn pin_respawn_to_current<'local>(
    api: &mut Api<'_, 'local>,
    player: &papermc::bukkit::Player<'local>,
    context: &'static str,
) -> eyre::Result<()> {
    if player.respawn_location(api)?.is_some() {
        return Ok(());
    }
    let current = player.location(api)?;
    player.set_respawn_location(api, &current, true)?;
    let x = current.block_x(api)?;
    let z = current.block_z(api)?;
    tracing::info!(x, z, context, "pinned player respawn location");
    Ok(())
}

/// Persist the player's current location as their respawn point on first join.
///
/// A returning player whose saved spawn was cleared (e.g. by `/spawnpoint reset`) gets their
/// current logoff position pinned, which is the least-surprising behavior.
pub(super) fn handle_player_join<'local>(
    _plugin: &mut NitwitteryPlugin,
    api: &mut Api<'_, 'local>,
    event: &PlayerJoinEventRef<'local>,
    _config: Config,
) -> eyre::Result<()> {
    let player = event.player(api)?;
    pin_respawn_to_current(api, &player, "join")
}

/// Persist the player's current location as their respawn point after every respawn.
///
/// Catches the post-rebias case: when `handle_player_respawn` overrides this respawn's
/// destination via `event.set_respawn_location`, Paper does not persist that as the new saved
/// spawn. After the respawn flow completes the player has no saved spawn, so on the next death
/// they'd fall back to world spawn. This handler pins the new location once respawn is finished.
pub(super) fn handle_player_post_respawn<'local>(
    _plugin: &mut NitwitteryPlugin,
    api: &mut Api<'_, 'local>,
    event: &PlayerPostRespawnEventRef<'local>,
    _config: Config,
) -> eyre::Result<()> {
    let player = event.player(api)?;
    pin_respawn_to_current(api, &player, "post-respawn")
}

/// Re-bias when vanilla detected a destroyed respawn block (bed broken or anchor depleted).
///
/// In every other case the saved respawn (bed, anchor, or a previously-pinned village) is used
/// as-is. Persisting the new village as the saved spawn happens later in
/// [handle_player_post_respawn]; calls to `Player#setRespawnLocation` during this event are
/// clobbered by Paper.
pub(super) fn handle_player_respawn<'local>(
    _plugin: &mut NitwitteryPlugin,
    api: &mut Api<'_, 'local>,
    event: &PlayerRespawnEventRef<'local>,
    config: Config,
) -> eyre::Result<()> {
    if event.respawn_reason(api)? != RespawnReason::Death {
        return Ok(());
    }
    if !event.is_missing_respawn_block(api)? {
        return Ok(());
    }

    let respawn = event.respawn_location(api)?;
    let Some(world) = respawn.world(api)? else {
        tracing::debug!("respawn: location has no world; skipping");
        return Ok(());
    };
    if world.environment(api)? != Environment::Normal {
        return Ok(());
    }

    let cx = respawn.x(api)?;
    let cz = respawn.z(api)?;

    // We are already on the main thread; locate inline, no scheduler hop.
    let coords = find_village_coords_inline(api, &world, cx, cz, config)?;

    let Some((vx, vy, vz)) = coords else {
        tracing::debug!("respawn: no village within reach; falling through");
        return Ok(());
    };

    let target = Location::new(api, &world, vx, vy, vz)?;
    event.set_respawn_location(api, &target)?;
    tracing::info!(
        x = vx,
        y = vy,
        z = vz,
        "re-biased respawn to village after bed loss",
    );
    Ok(())
}
