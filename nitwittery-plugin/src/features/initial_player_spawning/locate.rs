use rand::Rng;

/// Sample a point uniformly within a disc of `radius` centered at `(cx, cz)`.
///
/// Uses inverse-CDF on the radial coordinate (`r = R * sqrt(u)`) so the distribution is uniform
/// with respect to area, not radius.
#[allow(dead_code)]
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
