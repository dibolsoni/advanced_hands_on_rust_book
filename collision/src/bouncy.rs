use bevy::prelude::*;
use my_library::bevy_framework::Impulse;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
pub enum GamePhase {
    #[default]
    Loading,
    MainMenu,
    Bouncing,
    GameOver,
}

#[derive(Component)]
pub struct BouncyElement;

#[derive(Component)]
pub struct Ball;

#[derive(Resource, Default)]
pub struct CollisionTime {
    pub time: u128,
    pub checks: u32,
    pub fps: f64,
}

pub(crate) fn bounce_on_collision(
    entity: Entity,
    ball_a: Vec3,
    ball_b: Vec3,
    impulse: &mut MessageWriter<Impulse>,
) {
    let a_to_b = (ball_a - ball_b).normalize();
    impulse.write(Impulse {
        target: entity,
        amount: a_to_b / 8.0,
        absolute: false,
        source: 0,
    });
}
pub fn collisions(
    mut collision_time: ResMut<CollisionTime>,
    query: Query<(Entity, &Transform), With<Ball>>,
    mut impulse: MessageWriter<Impulse>,
) {
    let now = std::time::Instant::now();

    let mut n = 0;
    for (entity_a, ball_a) in query.iter() {
        query
            .iter()
            .filter(|(entity_b, _)| *entity_b != entity_a)
            .filter(|(_, ball_b)| {
                n += 1;
                ball_a.translation.distance(ball_b.translation) < (8.0 * 8.0)
            })
            .for_each(|(_, ball_b)| {
                bounce_on_collision(
                    entity_a,
                    ball_a.translation,
                    ball_b.translation,
                    &mut impulse,
                );
            });
    }
    collision_time.time = now.elapsed().as_millis();
    collision_time.checks = n;
}
