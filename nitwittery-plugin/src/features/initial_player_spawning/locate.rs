use jni::objects::JObject;
use jni::refs::Global;
use papermc::Api;
use papermc::bukkit::{Location, Structure, World};
use papermc::jobject_repr::JObjectRepr;
use rand::{Rng, RngExt};

use super::Config;

/// Sample a point uniformly within a disc of `radius` centered at `(cx, cz)`.
///
/// Uses inverse-CDF on the radial coordinate (`r = R * sqrt(u)`) so the distribution is uniform
/// with respect to area, not radius.
pub(super) fn random_point_in_disc(
    rng: &mut impl Rng,
    cx: f64,
    cz: f64,
    radius: f64,
) -> (f64, f64) {
    let u: f64 = rng.random();
    let theta: f64 = rng.random::<f64>() * std::f64::consts::TAU;
    let r = radius * u.sqrt();
    (cx + r * theta.cos(), cz + r * theta.sin())
}

/// Main-thread closure invoked via [Api::run_sync] from the async spawn handler.
///
/// Reconstructs a local [World] from the cross-thread [Global] reference, then delegates to the
/// inline locate logic shared with the respawn handler.
pub(super) fn find_village_location(
    api: &mut Api<'_, '_>,
    world_global: &Global<JObject<'static>>,
    cx: f64,
    cz: f64,
    config: Config,
) -> eyre::Result<Option<(f64, f64, f64)>> {
    let world_local = api.jni().new_local_ref(world_global)?;
    let world = unsafe { World::from_jobject(world_local) };
    find_village_coords_inline(api, &world, cx, cz, config)
}

/// Run the random-anchor + locate-village + surface-resolve loop.
///
/// Must be called on the main server thread because [World::locate_nearest_structure] with
/// `find_unexplored=true` may generate chunks.
pub(super) fn find_village_coords_inline<'local>(
    api: &mut Api<'_, 'local>,
    world: &World<'local>,
    cx: f64,
    cz: f64,
    config: Config,
) -> eyre::Result<Option<(f64, f64, f64)>> {
    let mut rng = rand::rng();
    for attempt in 0..config.max_attempts {
        let (ax, az) = random_point_in_disc(&mut rng, cx, cz, config.max_distance_from_spawn);
        // Anchor Y is irrelevant to the structure locator's horizontal search; 64 is a safe
        // placeholder.
        let anchor = Location::new(api, world, ax, 64.0, az)?;

        let Some(village) = locate_any_village(api, world, &anchor, config.locate_radius_chunks)?
        else {
            tracing::debug!(attempt, "no village near anchor; retrying");
            continue;
        };

        let vx = village.block_x(api)?;
        let vz = village.block_z(api)?;
        let surface_y = world.highest_block_y_at(api, vx, vz)?;
        return Ok(Some((
            f64::from(vx) + 0.5,
            f64::from(surface_y + 1),
            f64::from(vz) + 0.5,
        )));
    }
    Ok(None)
}

/// Locate the nearest of any of the five village biome variants relative to `anchor`.
///
/// Returns the closest result's [Location], or `None` if no variant has a result within
/// `radius_chunks`. The min-by-distance accumulator is inlined rather than extracted because each
/// [Location] is `!Copy` and reading its X/Z requires a `&mut Api` borrow, which would conflict
/// with collecting candidates into a `Vec` first.
fn locate_any_village<'local>(
    api: &mut Api<'_, 'local>,
    world: &World<'local>,
    anchor: &Location<'local>,
    radius_chunks: i32,
) -> eyre::Result<Option<Location<'local>>> {
    let ax = anchor.x(api)?;
    let az = anchor.z(api)?;

    let mut best: Option<(f64, Location<'local>)> = None;
    for variant in Structure::VILLAGES {
        // findUnexplored=true: required so fresh worlds bias correctly. All callers are on the
        // main thread (async handler routes through Api::run_sync; respawn handler is already
        // main).
        let Some(result) =
            world.locate_nearest_structure(api, anchor, variant, radius_chunks, true)?
        else {
            continue;
        };
        let loc = result.location(api)?;
        let lx = loc.x(api)?;
        let lz = loc.z(api)?;
        let dx = lx - ax;
        let dz = lz - az;
        let dist_sq = dx * dx + dz * dz;
        match &best {
            Some((b, _)) if *b <= dist_sq => {}
            _ => best = Some((dist_sq, loc)),
        }
    }
    Ok(best.map(|(_, loc)| loc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_point_in_disc_stays_within_radius() {
        let mut rng = rand::rng();
        let cx = 100.0;
        let cz = -50.0;
        let radius = 25.0;
        for _ in 0..10_000 {
            let (x, z) = random_point_in_disc(&mut rng, cx, cz, radius);
            let dx = x - cx;
            let dz = z - cz;
            let dist = (dx * dx + dz * dz).sqrt();
            assert!(
                dist <= radius + 1e-9,
                "sampled point at distance {dist} > radius {radius}",
            );
        }
    }

    #[test]
    fn random_point_in_disc_zero_radius() {
        let mut rng = rand::rng();
        let (x, z) = random_point_in_disc(&mut rng, 7.0, 11.0, 0.0);
        assert_eq!(x, 7.0);
        assert_eq!(z, 11.0);
    }
}
