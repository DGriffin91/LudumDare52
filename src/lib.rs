#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::f32::consts::TAU;

use assets::{fix_material_colors, AudioAssets, FontAssets, ModelAssets};
use audio::GameAudioPlugin;
use bevy::{
    ecs::{schedule::ShouldRun, system::EntityCommands},
    math::*,
    prelude::*,
    render::camera::Projection,
    window::{PresentMode, WindowMode, WindowResizeConstraints},
};
use bevy_asset_loader::prelude::{LoadingState, LoadingStateAppExt};

use bevy_mod_raycast::{RaycastMesh, RaycastSource};

use bevy_scene_hook::HookPlugin;
use board::GameBoard;

use items::{
    spawn_ore, spawn_outgoing_hats, Blobby, Dropoff, InitialPlayerResources,
    ResourcesAvailableToPlayer,
};
use iyes_loopless::prelude::*;
use player::{MyRaycastSet, PlayerState, Resources, R};

use rand::{seq::SliceRandom, Rng};
use rand_pcg::Pcg32;
use ridiculous_bevy_hot_reloading::HotReloadPlugin;
use ui::GameUI;
pub mod action;
pub mod assets;
pub mod audio;
pub mod board;
pub mod items;
pub mod player;
pub mod schedule;
pub mod ui;

/// #[no_mangle] Needed so libloading can find this entry point
#[no_mangle]
pub fn main() {
    let mut app = App::new();
    app.add_system(fix_material_colors)
        .add_loopless_state(GameState::AssetLoading)
        .add_loopless_state(PausedState::Unpaused)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::RunLevel)
                .with_collection::<FontAssets>()
                .with_collection::<ModelAssets>()
                .with_collection::<AudioAssets>(),
        );

    app.insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "LD52".to_string(),
                        width: 1280.0,
                        height: 720.0,
                        position: WindowPosition::Automatic,
                        resize_constraints: WindowResizeConstraints {
                            min_width: 960.0,
                            min_height: 480.0,
                            ..Default::default()
                        },
                        scale_factor_override: Some(1.0), //Needed for some mobile devices, but disables scaling
                        present_mode: PresentMode::AutoVsync,
                        resizable: true,
                        decorations: true,
                        cursor_visible: true,
                        mode: WindowMode::Windowed,
                        transparent: false,
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        ..default()
                    },
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..Default::default()
                }),
        )
        .add_plugin(HotReloadPlugin::default())
        .insert_resource(GameBoard::default())
        .insert_resource(RestartGame::default())
        .insert_resource(GameRng::default())
        .add_plugin(HookPlugin);

    app.add_plugin(GameUI).add_plugin(GameAudioPlugin);
    schedule::setup_schedule(&mut app);

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugin(bevy_web_resizer::Plugin);
    }

    app.add_enter_system(GameState::RunLevel, setup_level)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::RunLevel)
                //.with_system()
                .into(),
        );

    app.run();
}

#[derive(Resource, Deref, DerefMut)]
pub struct GameRng(pub Pcg32);

impl Default for GameRng {
    fn default() -> Self {
        GameRng(Pcg32::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7))
    }
}

#[derive(Component)]
pub struct Board;

/// set up a simple 3D scene
fn setup_level(
    mut com: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    model_assets: Res<ModelAssets>,
    mut b: ResMut<GameBoard>,
    mut rng: ResMut<GameRng>,
) {
    // plane
    com.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 24.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba(0.2, 0.2, 0.2, 0.0),
            perceptual_roughness: 0.4,
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        }),
        ..default()
    })
    .insert(Board)
    .insert(RaycastMesh::<MyRaycastSet>::default());

    // light
    com.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            //shadows_enabled: true,
            color: Color::rgb(1.0, 1.0, 1.0),
            ..default()
        },
        transform: Transform::from_translation(vec3(0.0, 5.0, 0.0)).with_rotation(
            Quat::from_euler(EulerRot::XYZ, TAU * 0.5, -TAU * 0.25, TAU * 0.25),
        ),
        ..default()
    });
    let side = 3.0;
    // camera
    com.spawn(Camera3dBundle {
        transform: (Transform::from_translation(vec3(48.0 + side, 48.0, 48.0 - side)))
            .looking_at(vec3(side, -2.0, -side), Vec3::Y),
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 16f32.to_radians(),
            ..default()
        }),
        ..default()
    })
    .insert(RaycastSource::<MyRaycastSet>::new());

    init_game(&mut com, &model_assets, &mut b, &mut rng);
}

pub fn init_game(
    com: &mut Commands,
    model_assets: &ModelAssets,
    b: &mut GameBoard,
    rng: &mut GameRng,
) {
    // Player initial resources
    com.spawn(ResourcesAvailableToPlayer)
        .insert(InitialPlayerResources)
        .insert(Resources::player_default());

    for _ in 0..20 {
        let x = rng.0.gen_range(3..20);
        let y = rng.0.gen_range(3..20);
        let kind = [R::CopperOre, R::LithiumOre, R::Sand]
            .choose(&mut rng.0)
            .unwrap();

        spawn_ore(com, model_assets, b, IVec2::new(x, y), *kind);
    }

    // Plastic
    spawn_ore(com, model_assets, b, IVec2::new(2, 2), R::Plastic);

    spawn_outgoing_hats(com, model_assets, b, IVec2::new(22, 22));

    com.spawn(SceneBundle {
        scene: model_assets.board.clone(),
        transform: Transform::from_translation(vec3(0.0, -0.1, 0.0)),
        ..default()
    });
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct RestartGame(bool);

fn restart_game(
    mut com: Commands,
    mut restart_game: ResMut<RestartGame>,
    mut player: ResMut<PlayerState>,
    mut b: ResMut<GameBoard>,
    //model_assets: Res<ModelAssets>,
    blobbies: Query<Entity, With<Blobby>>,
    resources: Query<Entity, With<Resources>>,
    dropoff: Query<Entity, With<Dropoff>>,
    scenes: Query<Entity, With<Handle<Scene>>>,
    mut rng: ResMut<GameRng>,
    model_assets: Res<ModelAssets>,
) {
    if **restart_game {
        **restart_game = false;
        for e in blobbies.iter() {
            com.entity(e).despawn_recursive();
        }
        for e in resources.iter() {
            com.entity(e).despawn_recursive();
        }
        for e in dropoff.iter() {
            com.entity(e).despawn_recursive();
        }
        for e in scenes.iter() {
            com.entity(e).despawn_recursive();
        }
        *b = GameBoard::default();

        let old_time_multiplier = player.time_multiplier;
        *player = PlayerState::default();
        player.time_multiplier = old_time_multiplier;
        init_game(&mut com, &model_assets, &mut b, &mut rng);
    }
}

pub fn basic_light(
    cmds: &mut EntityCommands,
    color: Color,
    intensity: f32,
    range: f32,
    radius: f32,
    trans: Vec3,
) {
    cmds.add_children(|parent| {
        parent.spawn(PointLightBundle {
            point_light: PointLight {
                color,
                intensity,
                range,
                radius,
                ..default()
            },
            transform: Transform::from_translation(trans),
            ..default()
        });
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    AssetLoading,
    RunLevel,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum PausedState {
    Unpaused,
    Paused,
}

pub fn game_state_asset_loading(state: Res<CurrentState<GameState>>) -> ShouldRun {
    if *state == CurrentState(GameState::AssetLoading) {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub fn game_state_run_level_unpaused(
    state: Res<CurrentState<GameState>>,
    paused_state: Res<CurrentState<PausedState>>,
) -> ShouldRun {
    if *state == CurrentState(GameState::RunLevel)
        && *paused_state == CurrentState(PausedState::Unpaused)
    {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
