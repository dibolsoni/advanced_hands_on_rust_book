use bevy::prelude::{
    Component, Entity, Local, Message, MessageReader, MessageWriter, Query, Res, Time, Transform,
    Vec3, With,
};
use std::collections::{HashMap, HashSet};

const PHYSICS_TICK_TIME: u128 = 33;

#[derive(Default)]
pub struct PhysicsTimer(u128);

#[derive(Message)]
pub struct PhysicsTick;

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Message)]
pub struct Impulse {
    pub target: Entity,
    pub amount: Vec3,
    pub absolute: bool,
    pub source: i32,
}

#[derive(Component)]
pub struct ApplyGravity;

pub fn physics_clock(
    mut clock: Local<PhysicsTimer>,
    time: Res<Time>,
    mut on_tick: MessageWriter<PhysicsTick>,
) {
    let ms_since_last_call = time.delta().as_millis();
    clock.0 += ms_since_last_call;
    if clock.0 >= PHYSICS_TICK_TIME {
        clock.0 = 0;
        on_tick.write(PhysicsTick);
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self(Vec3::ZERO)
    }
}

impl Velocity {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3 { x, y, z })
    }
}

pub fn sum_impulses(mut impulses: MessageReader<Impulse>, mut velocities: Query<&mut Velocity>) {
    let mut dedupe_by_source = HashMap::new();
    for impulse in impulses.read() {
        dedupe_by_source.insert(impulse.source, impulse);
    }
    let mut absolute = HashSet::new();
    for (_, impulse) in dedupe_by_source {
        if let Ok(mut velocity) = velocities.get_mut(impulse.target) {
            if absolute.contains(&impulse.target) {
                continue;
            }
            if impulse.absolute {
                velocity.0 = impulse.amount;
                absolute.insert(impulse.target);
            } else {
                velocity.0 += impulse.amount;
            }
        }
    }
}

pub fn apply_velocity(
    mut tick: MessageReader<PhysicsTick>,
    mut movement: Query<(&Velocity, &mut Transform)>,
) {
    for _tick in tick.read() {
        movement.iter_mut().for_each(|(velocity, mut transform)| {
            transform.translation += velocity.0;
        });
    }
}

pub fn apply_gravity(
    mut tick: MessageReader<PhysicsTick>,
    mut gravity: Query<&mut Velocity, With<ApplyGravity>>,
) {
    const GRAVITY_FORCE: f32 = 0.75;
    for _tick in tick.read() {
        gravity.iter_mut().for_each(|mut velocity| {
            velocity.0.y -= GRAVITY_FORCE;
        });
    }
}
