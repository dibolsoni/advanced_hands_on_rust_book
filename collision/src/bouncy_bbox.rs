use crate::bouncy::{bounce_on_collision, CollisionTime};
use bevy::math::Vec2;
use bevy::prelude::{Component, Entity, MessageWriter, Query, ResMut, Transform};
use my_library::bevy_framework::Impulse;
use std::time::Instant;
use crate::Rect2D;

#[derive(Component)]
pub struct AxisAlignedBoundingBox {
    half_size: Vec2,
}

impl AxisAlignedBoundingBox {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            half_size: Vec2::new(width / 2.0, height / 2.0),
        }
    }

    pub fn as_rect(&self, translate: Vec2) -> Rect2D {
        Rect2D::new(
            Vec2::new(
                translate.x - self.half_size.x,
                translate.y - self.half_size.y,
            ),
            Vec2::new(
                translate.x + self.half_size.x,
                translate.y + self.half_size.y,
            ),
        )
    }
}

pub fn collision_bbox(
    mut collision_time: ResMut<CollisionTime>,
    query: Query<(Entity, &Transform, &AxisAlignedBoundingBox)>,
    mut impulse: MessageWriter<Impulse>,
) {
    let now = Instant::now();

    let mut n = 0;
    for (entity_a, ball_a, box_a) in query.iter() {
        let box_a = box_a.as_rect(ball_a.translation.truncate());
        for (entity_b, ball_b, box_b) in query.iter() {
            if entity_a != entity_b {
                let box_b = box_b.as_rect(ball_b.translation.truncate());
                if box_a.intersect(&box_b) {
                    bounce_on_collision(
                        entity_a,
                        ball_a.translation,
                        ball_b.translation,
                        &mut impulse,
                    )
                }
                n += 1;
            }
        }
    }
    collision_time.time = now.elapsed().as_millis();
    collision_time.checks = n;
}
