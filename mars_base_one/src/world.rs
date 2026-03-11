use crate::GameElement;
use crate::Sprite;
use crate::Transform;
use bevy::prelude::{Commands, Component, Vec2};
use my_library::bevy_assets::{AssetStore, LoadedAssets};
use my_library::bevy_framework::{AxisAlignedBoundingBox, PhysicsPosition};
use my_library::{spawn_image, RandomNumberGenerator};

pub struct MarsWorld {
    solid: Vec<bool>,
    width: usize,
    height: usize,
}

#[derive(Component)]
pub struct Ground;

impl MarsWorld {
    fn mapidx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn find_random_closed_tile(&self, rng: &mut RandomNumberGenerator) -> (usize, usize) {
        loop {
            let x = rng.range(0..self.width);
            let y = rng.range(0..self.height);
            let idx = self.mapidx(x, y);
            if self.solid[idx] {
                return (x, y);
            }
        }
    }

    fn clear_tiles(&mut self, x: usize, y: usize) {
        for offset_x in -1..=1 {
            for offset_y in -1..=1 {
                let x = x as isize + offset_x;
                let y = y as isize + offset_y;
                if x > 0 && x < self.width as isize - 1 && y > 0 && y < self.height as isize {
                    let idx = self.mapidx(x as usize, y as usize);
                    self.solid[idx] = false;
                }
            }
        }
    }

    fn clear_line(&mut self, start: (usize, usize), end: (usize, usize)) {
        let (mut x, mut y) = (start.0 as f32, start.1 as f32);
        let (slope_x, slope_y) = (
            (end.0 as f32 - x) / self.width as f32,
            (end.1 as f32 - y) / self.height as f32,
        ); //(23)
        loop {
            //(24)
            let (tx, ty) = (x as usize, y as usize);
            if tx < 1 || tx > self.width - 1 || ty < 1 || ty > self.height - 1 {
                break; //(25)
            }
            if tx == end.0 && ty == end.1 {
                break; //(26)
            }
            self.clear_tiles(x as usize, y as usize);
            x += slope_x; //(27)
            y += slope_y;
        }
    }

    pub(crate) fn new(width: usize, height: usize, rng: &mut RandomNumberGenerator) -> Self {
        let mut result = Self {
            width,
            height,
            solid: vec![true; width * height],
        };

        // Set the center tile and surrounding tiles to be empty
        result.clear_tiles(width / 2, height / 2);

        // Blast some holes in the center
        let mut holes = vec![(width / 2, height / 2)];
        for _ in 0..10 {
            let x = rng.range(5..width - 5);
            let y = rng.range(5..height - 5);
            holes.push((x, y));
            result.clear_tiles(x, y);
            result.clear_tiles(x + 2, y);
            result.clear_tiles(x - 2, y);
            result.clear_tiles(x, y + 2);
            result.clear_tiles(x, y - 2);
        }

        // Cut a line between each hole
        for i in 0..holes.len() {
            let start = holes[i];
            let end = holes[(i + 1) % holes.len()];
            result.clear_line(start, end);
        }

        // Carve a borehole
        for y in height / 2..height {
            result.clear_tiles(width / 2, y);
        }

        // Outward diffusion
        let mut done = false;
        while !done {
            let start_tile = holes[rng.range(0..10)];
            let target = result.find_random_closed_tile(rng);
            let (mut x, mut y) = (start_tile.0 as f32, start_tile.1 as f32);
            let (slope_x, slope_y) = (
                (target.0 as f32 - x) / width as f32,
                (target.1 as f32 - y) / height as f32,
            );

            loop {
                if x < 1.0 || x > width as f32 || y < 1.0 || y > height as f32 {
                    break;
                }
                let tile_id = result.mapidx(x as usize, y as usize);
                if result.solid[tile_id] {
                    result.clear_tiles(x as usize, y as usize);
                    break;
                }
                x += slope_x;
                y += slope_y;
            }

            let solid_count = result.solid.iter().filter(|s| **s).count();
            let solid_percent = solid_count as f32 / (width * height) as f32;
            if solid_percent < 0.6 {
                done = true;
            }
        }

        result
    }

    pub(crate) fn spawn(&self, assets: &AssetStore, commands: &mut Commands, loaded_assets: &LoadedAssets) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.solid[y * self.width + x] {
                    let position = Vec2::new(
                        (x as f32 - self.width as f32 / 2.0) * 24.0,
                        (y as f32 - self.height as f32 / 2.0) * 24.0,
                    );

                    // spawn a solid block
                    spawn_image!(
                        assets,
                        commands,
                        "ground",
                        position.x,
                        position.y,
                        -1.0,
                        &loaded_assets,
                        GameElement,
                        Ground,
                        PhysicsPosition::new(Vec2::new(position.x, position.y,)),
                        AxisAlignedBoundingBox::new(24.0, 24.0)
                    );
                }
            }
        }
    }
}
