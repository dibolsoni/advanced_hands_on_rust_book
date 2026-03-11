use crate::world::{Ground, MarsWorld};
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use my_library::bevy_assets::{AssetManager, AssetStore, LoadedAssets};
use my_library::bevy_framework::{
    apply_gravity, apply_velocity, check_collisions, cleanup, physics_clock, sum_impulses,
    Animations, AxisAlignedBoundingBox, GameStatePlugin, Impulse, OnCollision, PhysicsPosition,
    PhysicsTick, StaticQuadTree, Velocity,
};
use my_library::egui::{EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use my_library::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

mod world;

static WORLD_READY: AtomicBool = AtomicBool::new(false);
static NEW_WORLD: Mutex<Option<MarsWorld>> = Mutex::new(None);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
pub enum GamePhase {
    #[default]
    Loading,
    MainMenu,
    WorldBuilding,
    Playing,
    GameOver,
}

#[derive(Component)]
pub struct GameElement;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MyCamera;

fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    add_phase!(app, GamePhase, GamePhase::Playing,
        start => [mars_setup],
        run => [
            end_game,
            physics_clock,
            sum_impulses,
            apply_gravity,
            apply_velocity,
            movement,
            terminal_velocity.after(apply_velocity),
            camera_follow.after(terminal_velocity),
            check_collisions::<Player, Ground>,
            bounce
        ],
        exit => [cleanup::<GameElement>]);

    add_phase!(app, GamePhase, GamePhase::WorldBuilding,
        start => [ spawn_builder ],
        run => [],
        exit => [ cleanup::<GameElement> ]
    );

    app.add_message::<Impulse>();
    app.add_message::<PhysicsTick>();
    app.add_message::<OnCollision<Player, Ground>>();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Mars Base One".to_string(),
            resolution: bevy::window::WindowResolution::new(1024.0 as u32, 768.0 as u32),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(RandomPlugin)
    .add_plugins(GameStatePlugin::new(
        GamePhase::MainMenu,
        GamePhase::WorldBuilding,
        GamePhase::GameOver,
    ))
    .add_plugins(
        AssetManager::new()
            .add_image("ship", "ship.png")?
            .add_image("ground", "ground.png")?,
    )
    .add_plugins(EguiPlugin::default())
    .add_systems(
        EguiPrimaryContextPass,
        show_builder.run_if(in_state(GamePhase::WorldBuilding)),
    )
    .insert_resource(Animations::new())
    .run();

    Ok(())
}

fn mars_setup(
    mut commands: Commands,
    assets: Res<AssetStore>,
    loaded_assets: Res<LoadedAssets>,
    mut rng: ResMut<RandomNumberGenerator>,
) {
    let cb = Camera2d;
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::WindowSize,
        scale: 2.0,
        ..OrthographicProjection::default_2d()
    });

    commands
        .spawn(cb)
        .insert(projection)
        .insert(GameElement)
        .insert(MyCamera);

    spawn_image!(
        assets,
        commands,
        "ship",
        0.0,
        0.0,
        1.0,
        &loaded_assets,
        GameElement,
        Player,
        Velocity::default(),
        PhysicsPosition::new(Vec2::new(0.0, 0.0)),
        AxisAlignedBoundingBox::new(24.0, 24.0)
    );

    let world = MarsWorld::new(200, 200, &mut rng);
    world.spawn(&assets, &mut commands, &loaded_assets);
    commands.insert_resource(StaticQuadTree::new(Vec2::new(10240.0, 7680.0), 6))
}

fn end_game(
    //mut state: ResMut<NextState<GamePhase>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(transform) = player_query.single() else {
        return;
    };
    if transform.translation.y < -384.0
        || transform.translation.y > 384.0
        || transform.translation.x < -512.0
        || transform.translation.x > 512.0
    {
        //state.set(GamePhase::GameOver);
    }
}

fn movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(Entity, &mut Transform), With<Player>>,
    mut impulses: MessageWriter<Impulse>,
) {
    let Ok((entity, mut transform)) = player_query.single_mut() else {
        return;
    };
    if keyboard.pressed(KeyCode::ArrowLeft) {
        transform.rotate(Quat::from_rotation_z(f32::to_radians(2.0))); // (4)
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        transform.rotate(Quat::from_rotation_z(f32::to_radians(-2.0))); // (5)
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        impulses.write(Impulse {
            // (6)
            target: entity,
            amount: transform.local_y().as_vec3(), // (7)
            absolute: false,
            source: 1,
        });
    }
}

fn terminal_velocity(mut player_query: Query<&mut Velocity, With<Player>>) {
    let Ok(mut velocity) = player_query.single_mut() else {
        return;
    };
    let v2 = velocity.0.truncate();
    if v2.length() > 5.0 {
        let v2 = v2.normalize() * 5.0;
        velocity.0.x = v2.x;
        velocity.0.y = v2.y;
    }
}

fn camera_follow(
    player_query: Query<&Transform, (With<Player>, Without<MyCamera>)>, // (8)
    mut camera_query: Query<&mut Transform, (With<MyCamera>, Without<Player>)>, // (9)
) {
    let Ok(player) = player_query.single() else {
        // (10)
        return;
    };
    let Ok(mut camera) = camera_query.single_mut() else {
        // (11)
        return;
    };
    camera.translation = Vec3::new(player.translation.x, player.translation.y, 10.0); // (12)
}

fn bounce(
    mut collections: MessageReader<OnCollision<Player, Ground>>,
    mut player_query: Query<&PhysicsPosition, With<Player>>,
    ground_query: Query<&PhysicsPosition, With<Ground>>,
    mut impulses: MessageWriter<Impulse>,
) {
    let mut bounce = Vec2::default();
    let mut entity = None;
    let mut bounces = 0;
    for collision in collections.read() {
        if let Ok(player) = player_query.single_mut() {
            if let Ok(ground) = ground_query.get(collision.entity_b) {
                entity = Some(collision.entity_a);
                let difference = player.start_frame - ground.start_frame;
                bounces += 1;
                bounce = difference
            }
        }
    }

    if bounce != Vec2::default() {
        bounce = bounce.normalize();
        impulses.write(Impulse {
            target: entity.unwrap(),
            amount: Vec3::new(bounce.x / bounces as f32, bounce.y / bounces as f32, 0.0),
            absolute: true,
            source: 2,
        });
    }
}

fn spawn_builder(mut commands: Commands) {
    println!("Spawning builder");

    commands.spawn((Camera2d, PrimaryEguiContext, GameElement));
    use std::sync::atomic::Ordering;

    WORLD_READY.store(false, Ordering::Relaxed);

    std::thread::spawn(|| {
        let mut rng = RandomNumberGenerator::new();
        let world = MarsWorld::new(200, 200, &mut rng);
        let mut world_lock = NEW_WORLD.lock().unwrap();
        *world_lock = Some(world);
        WORLD_READY.store(true, Ordering::Relaxed);
    });
}

fn show_builder(
    mut state: ResMut<NextState<GamePhase>>,
    mut egui_context: egui::EguiContexts,
) -> Result {
    egui::egui::Window::new("Performance").show(egui_context.ctx_mut()?, |ui| {
        ui.label("Building world...");
        if WORLD_READY.load(Ordering::Relaxed) {
            state.set(GamePhase::Playing);
        }
    });
    Ok(())
}
