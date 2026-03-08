mod bouncy;
mod bouncy_bbox;
mod bouncy_bbox_quadtree;

use crate::bouncy::{Ball, BouncyElement, CollisionTime, GamePhase};
use crate::bouncy_bbox::AxisAlignedBoundingBox;
use crate::bouncy_bbox_quadtree::{collision_bbox_quadtree, StaticQuadTree, QUAD_TREE_DEPTH};
use bevy::app::App;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::Sprite;
use bevy::prelude::Transform;
use bevy::prelude::*;
use bevy::prelude::{Res, Result};
use bevy::window::Window as AppWindow;
use bevy::DefaultPlugins;
use my_library::bevy_assets::{AssetManager, AssetResource, AssetStore};
use my_library::bevy_framework::{
    apply_velocity, cleanup, continual_parallax, physics_clock, sum_impulses, GameStatePlugin,
    Velocity,
};
use my_library::egui::egui::Window;
use my_library::egui::{
    egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext,
};
use my_library::*;

#[derive(Message)]
pub struct SpawnBall(pub u32);

#[derive(Resource)]
pub struct AppState {
    pub n_balls: u128,
}

#[derive(Debug, Clone)]
pub struct Rect2D {
    min: Vec2,
    max: Vec2,
}

impl Rect2D {
    fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    fn intersect(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
    fn quadrants(&self) -> Vec<Self> {
        let center = (self.min + self.max) / 2.0;
        vec![
            Self::new(self.min, center), // Top-left
            Self::new(
                Vec2::new(center.x, self.min.y),
                Vec2::new(self.max.x, center.y),
            ), // Top-Right
            Self::new(
                Vec2::new(self.min.x, center.y),
                Vec2::new(center.x, self.max.y),
            ), // Bottom-left
            Self::new(center, self.max), // Bottom-right
        ]
    }
}

pub fn setup(mut commands: Commands, mut state: ResMut<NextState<GamePhase>>) {
    commands.spawn((Camera2d::default(), PrimaryEguiContext));
    commands.insert_resource(CollisionTime::default());
    commands.insert_resource(StaticQuadTree::new(
        Vec2::new(1024.0, 768.0),
        QUAD_TREE_DEPTH,
    ));
    println!("Setup done");
}

pub fn show_performance(
    mut egui_context: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut collision_time: ResMut<CollisionTime>,
    app_state: Res<AppState>,
    mut spawn_ball: MessageWriter<SpawnBall>,
) -> Result {
    let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS);

    match fps {
        Some(fps) => {
            Window::new("Performance").show(egui_context.ctx_mut()?, |ui| {
                let fps = fps.average();
                let fps = fps.unwrap_or(0.0);
                let fps_text = format!("FPS: {fps:.1}");
                collision_time.fps = fps;
                let color = match fps as u32 {
                    0..=29 => egui::Color32::RED,
                    30..=59 => egui::Color32::GOLD,
                    _ => egui::Color32::GREEN,
                };
                ui.colored_label(color, &fps_text);
                ui.colored_label(
                    color,
                    &format!("Collision Time: {} ms", collision_time.time),
                );
                ui.label(&format!("Collision checks: {}", collision_time.checks));
                ui.label(&format!("# Balls: {}", app_state.n_balls));
                if ui.button("Add Ball").clicked() {
                    spawn_ball.write(SpawnBall(1));
                }
                if ui.button("Add 100 balls").clicked() {
                    spawn_ball.write(SpawnBall(100));
                }
            });
        }
        None => {
            println!("No fps");
        }
    }
    Ok(())
}

pub fn spawn_ball(
    mut commands: Commands,
    mut spawn_ball: MessageReader<SpawnBall>,
    mut rng: ResMut<RandomNumberGenerator>,
    assets: Res<AssetStore>,
    loaded_assets: AssetResource,
    mut app_state: ResMut<AppState>,
) {
    for sb in spawn_ball.read() {
        let qtd = sb.0;
        for _ in 0..qtd {
            let position = Vec3::new(rng.range(-512.0..512.0), rng.range(-384.0..384.0), 0.0);
            let velocity = Vec3::new(rng.range(-1.0..1.0), rng.range(-1.0..1.0), 0.0);
            spawn_image!(
                assets,
                commands,
                "green_ball",
                position.x,
                position.y,
                position.z,
                &loaded_assets,
                BouncyElement,
                Velocity::new(velocity.x, velocity.y, velocity.z),
                AxisAlignedBoundingBox::new(8.0, 8.0),
                Ball
            );
            app_state.n_balls += 1;
        }
        println!("Spawned {} balls", qtd);
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    app.add_message::<SpawnBall>();

    add_phase!(app, GamePhase, GamePhase::Bouncing,
        start => [ setup],
        run => [spawn_ball, collision_bbox_quadtree, continual_parallax, physics_clock, sum_impulses, apply_velocity],
        exit => [cleanup::<BouncyElement>]);

    app.add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(AppState { n_balls: 0 })
        .insert_resource(CollisionTime::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(AppWindow {
                title: "Naieve Collision".to_string(),
                resolution: bevy::window::WindowResolution::new(1024.0 as u32, 768.0 as u32),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RandomPlugin)
        .add_plugins(AssetManager::new().add_image("green_ball", "green_ball.png")?)
        .add_plugins(GameStatePlugin::new(
            GamePhase::MainMenu,
            GamePhase::Bouncing,
            GamePhase::GameOver,
        ))
        .add_plugins(EguiPlugin::default())
        .add_systems(
            EguiPrimaryContextPass,
            show_performance.run_if(in_state(GamePhase::Bouncing)),
        )
        .run();
    Ok(())
}
