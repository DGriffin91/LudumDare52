use std::time::Duration;

use bevy::{math::*, prelude::*};
use bevy_scene_hook::{HookedSceneBundle, SceneHook};
use iyes_loopless::{
    prelude::FixedTimesteps,
    state::{CurrentState, NextState},
};
use rkyv::{Archive, Deserialize, Serialize};

use bytecheck::CheckBytes;
use int_enum::IntEnum;

use crate::{
    assets::ModelAssets,
    board::GameBoard,
    items::{
        spawn_factory, spawn_outgoing_hats, Blobby, InitialPlayerResources, Item, Path,
        ResourcesAvailableToPlayer, Sellable,
    },
    player::{PlayerState, Resources, GAMESETTINGS, R},
    schedule::TIMESTEP_MILLI,
    PausedState, RestartGame,
};

pub fn process_actions(
    mut com: Commands,
    mut action_queue: ResMut<ActionQueue>,
    mut player: ResMut<PlayerState>,
    mut restart: ResMut<RestartGame>,
    model_assets: Res<ModelAssets>,
    mut b: ResMut<GameBoard>,
    //pref: Res<Preferences>,
    mut time_step_info: ResMut<FixedTimesteps>,
    paused_state: Res<CurrentState<PausedState>>,
    mut game_recorder: ResMut<GameRecorder>,
    mut blobbies: Query<&mut Blobby>,
    mut resources_for_player: Query<&mut Resources, With<ResourcesAvailableToPlayer>>,
    sellable: Query<Entity, With<Sellable>>,
    init_player_res: Query<Entity, With<InitialPlayerResources>>,
) {
    if game_recorder.play {
        while let Some((step, rec_actions)) = game_recorder.actions.0.get(game_recorder.play_head) {
            if *step as u64 == player.step {
                action_queue.0.push(Action::from_bytes(*rec_actions))
            } else {
                break;
            }
            game_recorder.play_head += 1;
        }
    }

    #[allow(unused_assignments)]
    #[allow(unused_mut)]
    let mut debug_build = false;

    #[cfg(debug_assertions)]
    {
        debug_build = true;
    }

    for action in action_queue.0.iter() {
        match action {
            Action::Empty => continue,
            Action::SellItem(x, y) => {
                let idx = b.ls_to_idx(ivec2(*x as i32, *y as i32));
                if let Some(entity) = b.board[idx] {
                    if sellable.get(entity).is_ok() {
                        if let Ok(item_res) = resources_for_player.get(entity) {
                            let item_res = item_res.clone();
                            if let Ok(mut resources) =
                                resources_for_player.get_mut(init_player_res.single())
                            {
                                *resources = resources.sum(&item_res);
                            }
                            b.destroy(&mut com, idx);
                        }
                    }
                }
            }
            Action::GameSpeedDec => {
                player.time_multiplier = (player.time_multiplier - 0.1).max(0.1);
                time_step_info.single_mut().step =
                    Duration::from_millis((TIMESTEP_MILLI as f64 / player.time_multiplier) as u64)
            }
            Action::GameSpeedInc => {
                player.time_multiplier = (player.time_multiplier + 0.1).min(10.0);
                time_step_info.single_mut().step =
                    Duration::from_millis((TIMESTEP_MILLI as f64 / player.time_multiplier) as u64)
            }
            Action::GamePause => {
                if *paused_state == CurrentState(PausedState::Paused) {
                    com.insert_resource(NextState(PausedState::Unpaused));
                } else {
                    com.insert_resource(NextState(PausedState::Paused));
                }
            }
            Action::RestartGame => {
                **restart = true;
            }
            Action::CheatCredits => {
                if debug_build {
                    if let Ok(mut resources) =
                        resources_for_player.get_mut(init_player_res.single())
                    {
                        for (_k, qty_v) in resources.0.iter_mut() {
                            *qty_v += 1000;
                        }
                    }
                }
            }
            Action::CheatLevel => {
                if debug_build {
                    player.level_time += 10.0;
                }
            }
            Action::MoveBlobby(x, y, id) => {
                for mut blobby in &mut blobbies {
                    if *id == blobby.id {
                        blobby.dest = Some(ivec2(*x as i32, *y as i32));
                        blobby.resource_pile = None;
                        blobby.drop_off = None;
                    }
                }
            }
            Action::Place(x, y, kind) => {
                let item = Item::from_int(*kind).unwrap();
                let ls_pos = ivec2(*x as i32, *y as i32);
                let idx = b.ls_to_idx(ls_pos);
                if b.board[idx].is_none()
                    && buy(&mut player, &item.cost(), &mut resources_for_player)
                {
                    let pos = b.ls_to_ws_vec3(b.idx_to_ls(idx));
                    let mut ecmds = com.spawn_empty();

                    match item {
                        Item::Blobby => {
                            player.blobby_count += 1;
                            ecmds
                                .insert(Path::default())
                                .insert(Blobby {
                                    id: player.blobby_count,
                                    dest: None,
                                    speed: GAMESETTINGS.blobby_speed,
                                    resource_pile: None,
                                    drop_off: None,
                                    going_to_pickup: true,
                                })
                                .insert(Resources::zero());

                            ecmds.insert(HookedSceneBundle {
                                scene: SceneBundle {
                                    scene: model_assets.blobby_guy.clone(),
                                    transform: Transform::from_translation(pos),
                                    ..default()
                                },
                                hook: SceneHook::new(move |_entity, _cmds| {}),
                            });
                        }
                        Item::CopperRefinery => {
                            spawn_factory(&mut com, &model_assets, &mut b, ls_pos, R::Copper);
                        }
                        Item::LithiumRefinery => {
                            spawn_factory(&mut com, &model_assets, &mut b, ls_pos, R::Lithium);
                        }
                        Item::GlassRefinery => {
                            spawn_factory(&mut com, &model_assets, &mut b, ls_pos, R::Glass);
                        }
                        Item::BatteryFactory => {
                            spawn_factory(&mut com, &model_assets, &mut b, ls_pos, R::Batteries);
                        }
                        Item::LittleHatFactory => {
                            spawn_factory(&mut com, &model_assets, &mut b, ls_pos, R::LittleHats);
                        }
                        Item::BigHatFactory => {
                            spawn_factory(&mut com, &model_assets, &mut b, ls_pos, R::BigHats);
                        }
                        Item::LightbulbFactory => {
                            spawn_factory(&mut com, &model_assets, &mut b, ls_pos, R::Lightbulbs);
                        }
                        Item::OutgoingHatsFactory => {
                            spawn_outgoing_hats(&mut com, &model_assets, &mut b, ls_pos)
                        }
                    };
                }
            }
        }
        //if let Some((turret, x, y)) = place_turret {
        //    let idx = b.ls_to_idx(ivec2(*x as i32, *y as i32));
        //    if !b.board[idx].filled && idx != b.ls_to_idx(b.start) {
        //        b.board[idx].filled = true; //Just temp fill so we can check
        //        let possible_path = b.path(b.start, b.dest);
        //        b.board[idx].filled = false; //Undo temp fill
        //        if possible_path.is_some() {
        //            let cost = turret.cost();
        //            if player.resources.plastic >= cost {
        //                player.resources.plastic -= cost;
        //                let pos = b.ls_to_ws_vec3(b.idx_to_ls(idx));
        //                b.board[idx].turret = Some(match turret {
        //                    Items::Blaster => {
        //                        Items::spawn_blaster_turret(&mut com, pos, &model_assets, &pref)
        //                    }
        //                    Items::Laser => Items::spawn_laser_continuous_turret(
        //                        &mut com,
        //                        pos,
        //                        &model_assets,
        //                        &pref,
        //                    ),
        //                    Items::Wave => {
        //                        Items::spawn_shockwave_turret(&mut com, pos, &model_assets, &pref)
        //                    }
        //                    Items::Blobby => todo!(),
        //                });
        //
        //                b.board[idx].filled = true;
        //            }
        //        }
        //    }
        //}
    }

    if !game_recorder.disable_rec {
        action_queue.0.retain_mut(|action| {
            !matches!(
                action,
                Action::Empty
                    | Action::GameSpeedDec
                    | Action::GameSpeedInc
                    | Action::GamePause
                    | Action::RestartGame
            )
        });

        for action in action_queue.iter() {
            game_recorder
                .actions
                .0
                .push((player.step as u32, action.to_bytes()));
        }
    }

    action_queue.0 = Vec::new(); // Clear action queue
}

fn buy(
    player: &mut PlayerState,
    cost: &Resources,
    resources_for_player: &mut Query<&mut Resources, With<ResourcesAvailableToPlayer>>,
) -> bool {
    if player
        .combined_resources
        .take(cost, &mut Resources::zero(), true)
    {
        let mut cost = cost.clone();
        for mut r in resources_for_player {
            let mut dest = Resources::zero();
            if r.take(&cost, &mut dest, false) {
                break;
            }
            cost.take(&dest, &mut Resources::zero(), false);
        }
        true
    } else {
        false
    }
}

#[derive(Archive, Deserialize, Serialize, Clone, Eq, PartialEq, Default, Debug)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes))]
pub struct ActionRecording(Vec<(u32, [u8; 4])>);

#[derive(Resource, Default)]
pub struct GameRecorder {
    pub actions: ActionRecording,
    pub disable_rec: bool,
    pub play: bool,
    pub play_head: usize,
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct ActionQueue(pub Vec<Action>);

#[derive(Archive, Deserialize, Serialize, Clone, Copy, Eq, PartialEq, Debug)]
#[archive_attr(derive(CheckBytes))]
pub enum Action {
    Empty,
    SellItem(u8, u8),
    GameSpeedDec,
    GameSpeedInc,
    GamePause,
    RestartGame,
    CheatCredits,
    CheatLevel,
    MoveBlobby(u8, u8, u8),
    Place(u8, u8, u8),
}

impl Action {
    #[rustfmt::skip]
    pub fn to_bytes(&self) -> [u8; 4] {
        match self {
            Action::Empty                                 => [0,   0,  0,  0],
            Action::SellItem(x, y)              => [1,  *x, *y,  0],
            Action::GameSpeedDec                          => [2,   0,  0,  0],
            Action::GameSpeedInc                          => [3,   0,  0,  0],
            Action::GamePause                             => [4,   0,  0,  0],
            Action::RestartGame                           => [5,   0,  0,  0],
            Action::CheatCredits                          => [6,   0,  0,  0],
            Action::CheatLevel                            => [7,   0,  0,  0],
            Action::MoveBlobby(x, y, id)   => [8,  *x, *y, *id],
            Action::Place(x, y, id)        => [9,  *x, *y, *id],
        }
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        let x = bytes[1];
        let y = bytes[2];
        let id = bytes[3];
        match bytes[0] {
            0 => Action::Empty,
            1 => Action::SellItem(x, y),
            2 => Action::GameSpeedDec,
            3 => Action::GameSpeedInc,
            4 => Action::GamePause,
            5 => Action::RestartGame,
            6 => Action::CheatCredits,
            7 => Action::CheatLevel,
            8 => Action::MoveBlobby(x, y, id),
            9 => Action::Place(x, y, id),
            _ => Action::Empty,
        }
    }
}
