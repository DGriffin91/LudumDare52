use std::time::Duration;

use bevy::prelude::*;
use bevy_mod_raycast::{DefaultRaycastingPlugin, RaycastSystem};
use bevy_system_graph::SystemGraph;
use iyes_loopless::prelude::*;

use crate::{
    action::*, game_state_run_level_unpaused, items::*, player::*, restart_game, GameState,
};

pub const TIMESTEP_MILLI: u64 = 16;
pub const TIMESTEP: f32 = 0.016;
pub const TIMESTEP_SEC_F64: f64 = 0.016;

pub(crate) fn setup_schedule(app: &mut bevy::prelude::App) {
    let mut fixed_update_stage = SystemStage::parallel();

    app.add_system_set(
        ConditionSet::new()
            .label("mouse_interact")
            .run_in_state(GameState::RunLevel)
            .with_system(mouse_interact)
            .into(),
    );

    app.add_plugin(DefaultRaycastingPlugin::<MyRaycastSet>::default())
        .insert_resource(PlayerState::default())
        .add_enter_system(GameState::RunLevel, setup_player)
        .add_system_to_stage(
            CoreStage::First,
            update_raycast_with_cursor.before(RaycastSystem::BuildRays::<MyRaycastSet>),
        );

    fixed_update_stage.add_system_set(
        Into::<SystemSet>::into(SystemGraph::new().root(set_level).graph())
            .with_run_criteria(game_state_run_level_unpaused)
            .label("STEP PLAYER")
            .before("STEP BLOBBY"),
    );

    fixed_update_stage.add_system_set(
        Into::<SystemSet>::into(
            SystemGraph::new()
                .root(receive_plastic)
                .then(blobby_get_resource)
                .then(blobby_put_resource)
                .then(update_blobby_paths)
                .then(move_blobby_along_path)
                .then(process_factories)
                .then(update_player_resources)
                .then(hats_objective)
                //.then(debug_show_blobby_path)
                .graph(),
        )
        .with_run_criteria(game_state_run_level_unpaused)
        .label("STEP BLOBBY"),
    );

    app.insert_resource(ActionQueue::default());
    app.insert_resource(GameRecorder::default());
    fixed_update_stage.add_system_set(
        ConditionSet::new()
            .run_in_state(GameState::RunLevel)
            .label("STEP ACTION")
            .after("STEP BLOBBY")
            .with_system(process_actions)
            .into(),
    );

    fixed_update_stage.add_system_set(
        ConditionSet::new()
            .run_in_state(GameState::RunLevel)
            .label("STEP RESTART GAME")
            .after("STEP ACTION")
            .with_system(restart_game)
            .into(),
    );

    app.add_stage_after(
        CoreStage::Update,
        "my_fixed_update",
        FixedTimestepStage::new(Duration::from_millis(TIMESTEP_MILLI), "main")
            .with_stage(fixed_update_stage),
    );
}
