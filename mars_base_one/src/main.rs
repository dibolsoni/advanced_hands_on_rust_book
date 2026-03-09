use bevy::prelude::*;
use my_library::bevy_assets::{AssetManager, AssetStore, LoadedAssets};
use my_library::bevy_framework::{
    apply_gravity, apply_velocity, cleanup, physics_clock, sum_impulses, Animations, ApplyGravity,
    GameStatePlugin, Impulse, PhysicsPosition, PhysicsTick, Velocity,
};
use my_library::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
pub enum GamePhase {
    #[default]
    Loading,
    MainMenu,
    Playing,
    GameOver,
}

#[derive(Component)]
pub struct GameElement;

#[derive(Component)]
pub struct Player;

fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    add_phase!(app, GamePhase, GamePhase::Playing,
        start => [setup],
        run => [
            end_game,
            physics_clock,
            sum_impulses,
            apply_gravity,
            apply_velocity,
            movement,
            terminal_velocity.after(apply_velocity)
        ],
        exit => [cleanup::<GameElement>]);

    app.add_message::<Impulse>();
    app.add_message::<PhysicsTick>();

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
        GamePhase::Playing,
        GamePhase::GameOver,
    ))
    .add_plugins(AssetManager::new().add_image("ship", "ship.png")?)
    .insert_resource(Animations::new())
    .run();

    Ok(())
}

fn setup(mut commands: Commands, assets: Res<AssetStore>, loaded_assets: Res<LoadedAssets>) {
    commands.spawn(Camera2d).insert(GameElement);

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
        ApplyGravity
    );
}

fn end_game(
    mut state: ResMut<NextState<GamePhase>>,
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
        state.set(GamePhase::GameOver);
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
        transform.rotate(Quat::from_rotation_z(f32::to_radians(2.0)));
    }

    if keyboard.pressed(KeyCode::ArrowRight) {
        transform.rotate(Quat::from_rotation_z(f32::to_radians(-2.0)));
    }

    if keyboard.pressed(KeyCode::ArrowUp) {
        impulses.write(Impulse {
            target: entity,
            amount: transform.local_y().as_vec3(),
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
