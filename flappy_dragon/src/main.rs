use bevy::prelude::*;
use my_library::bevy_assets::{AssetManager, AssetResource, AssetStore, LoadedAssets};
use my_library::bevy_framework::{cleanup, GameStatePlugin};
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

    add_phase!(app, GamePhase, GamePhase::Flapping,
        start => [ setup ],
        run => [gravity, flap, clamp, move_walls, hit_wall],
        exit => [cleanup::<FlappyElement>]
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
            .add_sound("dragonflap", "dragonflap.ogg")?
            .add_sound("crash", "crash.ogg")?,
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
    commands.spawn(Camera2d::default()).insert(FlappyElement); //<callout id="flappy.basics.2d_camera" />
    spawn_image!(
        assets,
        commands,
        "flappy_dragon",
        -490.0,
        0.0,
        1.0,
        &loaded_assets,
        Flappy { gravity: 0.0 },
        FlappyElement
    );
    build_wall(&mut commands, &assets, &loaded_assets, rng.range(-5..5)); //<callout id="flappy.basics.build_wall" />
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
                1.0,
                &loaded_assets,
                Obstacle,
                FlappyElement
            );
        }
    }
}
//END: build_wall

//START: gravity
fn gravity(mut query: Query<(&mut Flappy, &mut Transform)>) {
    if let Ok((mut flappy, mut transform)) = query.single_mut() {
        //<callout id="flappy.basics.get_flappy" />
        flappy.gravity += 0.1; //<callout id="flappy.basics.inc_gravity" />
        transform.translation.y -= flappy.gravity; //<callout id="flappy.basics.dec_pos" />
    }
}
//END: gravity

//START: flap
fn flap(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Flappy>,
    assets: Res<AssetStore>,
    loaded_assets: Res<LoadedAssets>,
    mut commands: Commands,
) {
    if keyboard.pressed(KeyCode::Space) {
        if let Ok(mut flappy) = query.single_mut() {
            flappy.gravity = -5.0; //<callout id="flappy.basics.flap" />
            assets.play("dragonflap", &mut commands, &loaded_assets);
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
    mut query: Query<&mut Transform, With<Obstacle>>,
    delete: Query<Entity, With<Obstacle>>,
    assets: Res<AssetStore>,
    loaded_assets: AssetResource,
    rng: ResMut<RandomNumberGenerator>,
) {
    let mut rebuild = false;
    for mut transform in query.iter_mut() {
        transform.translation.x -= 4.0;
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
    player: Query<&Transform, With<Flappy>>, //<callout id="flappy.basics.find_player" />
    walls: Query<&Transform, With<Obstacle>>, //<callout id="flappy.basics.find_walls" />
    mut state: ResMut<NextState<GamePhase>>,
    assets: Res<AssetStore>,
    loaded_assets: Res<LoadedAssets>,
    mut commands: Commands,
) {
    if let Ok(player) = player.single() {
        //<callout id="flappy.basics.just_player" />
        for wall in walls.iter() {
            //<callout id="flappy.basics.all_walls" />
            let distance = player.translation.distance(wall.translation); //<callout id="flappy.basics.distance" />
            if distance < 32.0 {
                assets.play("crash", &mut commands, &loaded_assets);
                state.set(GamePhase::GameOver);
            }
        }
    }
}
//END: hit_wall
