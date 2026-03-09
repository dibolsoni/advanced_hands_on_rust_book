use bevy::prelude::*;
use my_library::bevy_assets::{AssetManager, AssetResource, AssetStore, LoadedAssets};
use my_library::bevy_framework::{apply_gravity, apply_velocity, check_collisions, cleanup, physics_clock, sum_impulses, AnimationFrame, AnimationOption, Animations, ApplyGravity, AxisAlignedBoundingBox, ContinualParallax, GameStatePlugin, Impulse, OnCollision, PerFrameAnimation, PhysicsPosition, PhysicsTick, StaticQuadTree, Velocity};
use my_library::bevy_framework::{continual_parallax, cycle_animations, AnimationCycle};
use my_library::egui::PrimaryEguiContext;
use my_library::*;

//START: components
#[derive(Component)]
struct Flappy {
    //<callout id="flappy.basics.flappy" />
    gravity: f32, //<callout id="flappy.basics.gravity" />
}

#[derive(Component)]
struct Obstacle; //<callout id="flappy.basics.obstacle" />

#[derive(Component)]
struct FlappyElement;
//END: components

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
    #[default]
    Loading,
    MainMenu,
    Flapping,
    GameOver,
}

//START: main
fn main() -> anyhow::Result<()> {
    let mut app = App::new();
    app.add_message::<Impulse>();
    app.add_message::<PhysicsTick>();
    app.add_message::<OnCollision<Flappy, Obstacle>>();
    add_phase!(app, GamePhase, GamePhase::Flapping,
      start => [ setup ],
      run => [ flap, clamp, move_walls, hit_wall, cycle_animations,
        continual_parallax, physics_clock, sum_impulses, apply_gravity,
        apply_velocity, check_collisions::<Flappy, Obstacle>, rotate],
      exit => [ cleanup::<FlappyElement> ]
    );

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            //<callout id="flappy.basics.window_desc" />
            title: "Flappy Dragon - Bevy Edition".to_string(),
            resolution: bevy::window::WindowResolution::new(1024.0 as u32, 768.0 as u32),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(RandomPlugin)
    .add_plugins(GameStatePlugin::new(
        GamePhase::MainMenu,
        GamePhase::Flapping,
        GamePhase::GameOver,
    ))
    .add_plugins(
        AssetManager::new()
            .add_image("flappy_dragon", "flappy_dragon.png")?
            .add_image("wall", "wall.png")?
            .add_image("bg_static", "rocky-far-mountains.png")?
            .add_image("bg_far", "rocky-nowater-far.png")?
            .add_image("bg_mid", "rocky-nowater-mid.png")?
            .add_image("bg_close", "rocky-nowater-close.png")?
            .add_sound("dragonflap", "dragonflap.ogg")?
            .add_sound("crash", "crash.ogg")?
            .add_spritesheet(
                "flappy_sprite_sheet",
                "flappy_sprite_sheet.png",
                62.0,
                65.0,
                4,
                1,
            )?,
    )
    .insert_resource(
        Animations::new()
            .with_animation(
                "Straight and Level",
                PerFrameAnimation::new(vec![
                    AnimationFrame::new(2, 500, vec![AnimationOption::NextFrame]),
                    AnimationFrame::new(3, 500, vec![AnimationOption::GoToFrame(0)]),
                ]),
            )
            .with_animation(
                "Flapping",
                PerFrameAnimation::new(vec![
                    AnimationFrame::new(
                        0,
                        66,
                        vec![
                            AnimationOption::NextFrame,
                            AnimationOption::PlaySound("dragonflap".to_string()),
                        ],
                    ),
                    AnimationFrame::new(1, 66, vec![AnimationOption::NextFrame]),
                    AnimationFrame::new(2, 66, vec![AnimationOption::NextFrame]),
                    AnimationFrame::new(3, 66, vec![AnimationOption::NextFrame]),
                    AnimationFrame::new(2, 66, vec![AnimationOption::NextFrame]),
                    AnimationFrame::new(
                        1,
                        66,
                        vec![AnimationOption::SwitchToAnimation(
                            "Straight and Level".to_string(),
                        )],
                    ),
                ]),
            ),
    )
    .run();
    Ok(())
}
//END: main

//START: setup
fn setup(
    mut commands: Commands,
    mut rng: ResMut<RandomNumberGenerator>,
    assets: Res<AssetStore>,
    loaded_assets: AssetResource,
) {
    commands
        .spawn((Camera2d::default(), PrimaryEguiContext))
        .insert(FlappyElement); //<callout id="flappy.basics.2d_camera" />
    commands.insert_resource(StaticQuadTree::new(Vec2::new(1024.0, 768.0), 4));
    spawn_animated_sprite!(
        assets,
        commands,
        "flappy_sprite_sheet",
        -490.0,
        0.0,
        10.0,
        "Straight and Level",
        Flappy { gravity: 0.0 },
        FlappyElement,
        Velocity::default(),
        ApplyGravity,
        AxisAlignedBoundingBox::new(62.0, 65.0),
        PhysicsPosition::new(Vec2::new(-490.0, 0.0))
    );
    build_wall(&mut commands, &assets, &loaded_assets, rng.range(-5..5));
    spawn_image!(
        assets,
        commands,
        "bg_static",
        0.0,
        0.0,
        1.0,
        &loaded_assets,
        FlappyElement
    );
    spawn_image!(
        assets,
        commands,
        "bg_far",
        0.0,
        0.0,
        2.0,
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(1280.0, 66, Vec2::new(1.0, 0.0))
    );
    spawn_image!(
        assets,
        commands,
        "bg_far",
        1280.0,
        0.0,
        2.0,
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(1280.0, 66, Vec2::new(1.0, 0.0))
    );
    spawn_image!(
        assets,
        commands,
        "bg_mid",
        0.0,
        0.0,
        3.0,
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(1280.0, 33, Vec2::new(1.0, 0.0))
    );
    spawn_image!(
        assets,
        commands,
        "bg_mid",
        1280.0,
        0.0,
        3.0,
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(1280.0, 33, Vec2::new(1.0, 0.0))
    );
    spawn_image!(
        assets,
        commands,
        "bg_close",
        0.0,
        0.0,
        4.0,
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(1280.0, 16, Vec2::new(2.0, 0.0))
    );
    spawn_image!(
        assets,
        commands,
        "bg_close",
        1280.0,
        0.0,
        4.0,
        &loaded_assets,
        FlappyElement,
        ContinualParallax::new(1280.0, 16, Vec2::new(2.0, 0.0))
    );
}
//END: setup

// START: build_wall
fn build_wall(
    commands: &mut Commands,
    assets: &AssetStore,
    loaded_assets: &LoadedAssets,
    gap_y: i32,
) {
    for y in -12..=12 {
        if y < gap_y - 4 || y > gap_y + 4 {
            spawn_image!(
                assets,
                commands,
                "wall",
                512.0,
                y as f32 * 32.0,
                10.0,
                &loaded_assets,
                Obstacle,
                FlappyElement,
                Velocity::new(-8.0, 0.0, 0.0),
                AxisAlignedBoundingBox::new(32.0, 32.0),
                PhysicsPosition::new(Vec2::new(512.0, y as f32 * 32.0))
            );
        }
    }
}

fn flap(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut AnimationCycle)>,
    mut impulse: MessageWriter<Impulse>,
) {
    if keyboard.pressed(KeyCode::Space) {
        if let Ok((flappy, mut animation)) = query.single_mut() {
            impulse.write(Impulse {
                target: flappy,
                amount: Vec3::Y,
                absolute: false,
                source: 0,
            });
            animation.switch("Flapping");
        }
    }
}
//END: flap

//START: clamp
fn clamp(mut query: Query<&mut Transform, With<Flappy>>, mut state: ResMut<NextState<GamePhase>>) {
    if let Ok(mut transform) = query.single_mut() {
        if transform.translation.y > 384.0 {
            transform.translation.y = 384.0; //<callout id="flappy.basics.at_the_top" />
        } else if transform.translation.y < -384.0 {
            state.set(GamePhase::GameOver)
        }
    }
}
//END: clamp

//START: move_wall1
fn move_walls(
    mut commands: Commands,
    query: Query<&Transform, With<Obstacle>>,
    delete: Query<Entity, With<Obstacle>>,
    assets: Res<AssetStore>,
    loaded_assets: AssetResource,
    mut rng: ResMut<RandomNumberGenerator>,
) {
    let mut rebuild = false;
    for transform in query.iter() {
        if transform.translation.x < -530.0 {
            rebuild = true; //<callout id="flappy.basics.need_rebuild" />
        }
    }
    //END: move_wall1
    //START: move_wall2
    if rebuild {
        for entity in delete.iter() {
            commands.entity(entity).despawn();
        }
        build_wall(&mut commands, &assets, &loaded_assets, rng.range(-5..5));
    }
}
//END: move_wall2

//START: hit_wall
fn hit_wall(
    mut collisions: MessageReader<OnCollision<Flappy, Obstacle>>,
    mut state: ResMut<NextState<GamePhase>>,
    assets: Res<AssetStore>,
    loaded_assets: Res<LoadedAssets>,
    mut commands: Commands,
) {
    for _collision in collisions.read() {
        assets.play("crash", &mut commands, &loaded_assets);
        let _ = state.set(GamePhase::GameOver);
    }
}
//END: hit_wall

pub fn rotate(mut physics_position: Query<(&mut PhysicsPosition, &mut Transform), With<Flappy>>) {
    physics_position
        .iter_mut()
        .for_each(|(position, mut transform)| {
            if position.start_frame != position.end_frame {
                let start = position.start_frame;
                let end = position.end_frame;
                let angle = end.angle_to(start) * 10.0;
                transform.rotation = Quat::from_rotation_z(angle);
            }
        })
}
