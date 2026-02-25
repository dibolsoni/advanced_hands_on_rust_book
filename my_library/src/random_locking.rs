use bevy::prelude::Resource;
use rand::distr::uniform::SampleRange;
use rand::prelude::*;
use std::sync::Mutex;
use rand_pcg::rand_core::SeedableRng;

#[cfg(all(not(feature = "xorshift"), not(feature = "pcg")))]
pub type RngCore = rand::prelude::StdRng;
#[cfg(feature = "xorshift")]
pub type RngCore = rand_xorshift::XorShiftRng;
#[cfg(feature = "pcg")]
pub type RngCore = rand_pcg::Pcg64;

#[derive(Resource)]
pub struct RandomNumberGenerator {
    rng: Mutex<RngCore>,
}

impl RandomNumberGenerator {
    pub fn new() -> Self {
        let mut seeder = rand::rng();
        Self {
            rng: Mutex::new(RngCore::from_rng(&mut seeder)),
        }
    }

    pub fn seeded(seed: u64) -> Self {
        Self {
            rng: Mutex::new(RngCore::seed_from_u64(seed)),
        }
    }

    pub fn range<T>(&self, range: impl SampleRange<T>) -> T
    where
        T: rand::distr::uniform::SampleUniform + PartialOrd,
    {
        let mut lock = self.rng.lock().unwrap();
        lock.random_range(range)
    }

    pub fn next<T>(&self) -> T
    where
        rand::distr::StandardUniform: Distribution<T>,
    {
        let mut lock = self.rng.lock().unwrap();
        lock.random()
    }
}

impl Default for RandomNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RandomPlugin;
impl bevy::app::Plugin for RandomPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(RandomNumberGenerator::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_bounds() {
        let rng = RandomNumberGenerator::new();
        for _ in 0..1000 {
            let n = rng.range(1..10);
            assert!(n > 0, "n = {}", n);
            assert!(n < 10, "n = {}", n);
        }
    }

    #[test]
    fn test_reproducibility() {
        let rng = (
            RandomNumberGenerator::seeded(0),
            RandomNumberGenerator::seeded(0),
        );
        (0..1000).for_each(|_| {
            assert_eq!(
                rng.0.range(u32::MIN..u32::MAX),
                rng.1.range(u32::MIN..u32::MAX)
            )
        });
    }

    #[test]
    fn test_next_types() {
        let rng = RandomNumberGenerator::new();
        let _: i32 = rng.next();
        let _ = rng.next::<f32>();
    }

    #[test]
    fn test_range_float() {
        let rng = RandomNumberGenerator::new();
        for _ in 0..1000 {
            let n = rng.range(-5000.0f32..5000.0f32);
            assert!(n.is_finite());
            assert!(n > -5000.0);
            assert!(n < 5000.0);
        }
    }
}
