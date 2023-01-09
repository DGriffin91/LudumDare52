use bevy::math::*;
use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_egui::egui::Ui;
use bevy_egui::{egui::FontDefinitions, *};
use iyes_loopless::prelude::ConditionSet;

use crate::action::Action;
use crate::action::ActionQueue;
use crate::action::ActionRecording;
use crate::action::GameRecorder;
use crate::audio::AudioEvents;
use crate::audio::MUSIC_LEVEL_CHANGED;
//use crate::audio::SFX_LEVEL_CHANGED;

use crate::GameState;

use crate::items::Dropoff;
use crate::items::Item;
use crate::items::OutgoingHats;
use crate::player::PlayerState;
use crate::player::R;

pub struct GameUI;
impl Plugin for GameUI {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(Preferences::default())
            .add_system_set(
                ConditionSet::new()
                    .before("mouse_interact")
                    .run_in_state(GameState::RunLevel)
                    .with_system(ui_sidebar)
                    .with_system(ui_sidebar_left)
                    .into(),
            )
            .add_startup_system(setup_fonts);
    }
}

pub const SELECTED_COLOR: Color32 = Color32::from_rgb(255 / 2, 160 / 2, 98 / 2);
pub const DESELECTED_COLOR: Color32 = Color32::from_rgb(255 / 8, 160 / 8, 98 / 8);
pub const TEXT_COLOR: Color32 = Color32::from_rgb(255, 200, 145);
pub const TEXT_COLOR2: Color32 = Color32::from_rgb(140, 170, 170);

fn select_button(ui: &mut egui::Ui, text: &str, selected: bool) -> egui::Response {
    ui.add(egui::Button::new(text).fill(if selected {
        SELECTED_COLOR
    } else {
        DESELECTED_COLOR
    }))
}

fn ui_buy_button(
    ctx: &mut EguiContext,
    ui: &mut Ui,
    message: &str,
    item: Item,
    player: &mut PlayerState,
) {
    let response = select_button(ui, message, player.item_to_place == Some(item));
    if response.clicked() {
        player.item_to_place = Some(item);
        player.sell_mode = false;
        player.selected_entity = None;
    }
    if response.hovered() {
        egui::show_tooltip(
            ctx.ctx_mut(),
            egui::Id::new(format!("cost_hover{}", message)),
            |ui| {
                let mut style = ui.style_mut();
                style.visuals.override_text_color = Some(TEXT_COLOR2);
                ui.label("BUILD COST");
                item.cost()
                    .draw(&format!("item_cost{}", message), ui, false, false, false);
                if let Some((input, output)) = item.recipe() {
                    ui.label("INPUT");
                    input.draw(&format!("input_cost{}", message), ui, false, false, false);
                    ui.label("OUTPUT");
                    output.draw(&format!("output_cost{}", message), ui, false, false, false);
                }
            },
        );
    }
}

fn ui_sidebar(
    mut ctx: ResMut<EguiContext>,
    mut player: ResMut<PlayerState>,
    mut windows: ResMut<Windows>,
    mut pref: ResMut<Preferences>,
    mut audio_events: ResMut<AudioEvents>,
    mut action_queue: ResMut<ActionQueue>,
    mut game_recorder: ResMut<GameRecorder>,
    //mut rec_string: Local<String>,
    mut player_last_dead: Local<bool>,
    outgoing_hats: Query<&Dropoff, With<OutgoingHats>>,
) {
    let mut _player_died_this_frame = false;
    if !*player_last_dead && !player.alive() {
        _player_died_this_frame = true;
        *player_last_dead = true;
    }

    let window = windows.get_primary_mut().unwrap();
    let my_frame = egui::containers::Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 200),
        stroke: egui::Stroke::NONE,
        ..default()
    };

    egui::SidePanel::right("right_panel")
        .frame(my_frame)
        .resizable(false)
        .min_width(window.width() * 0.17)
        .default_width(window.width() * 0.17)
        .show_separator_line(false)
        .show(ctx.clone().ctx_mut(), |ui| {
            let mut style = ui.style_mut();
            style.visuals.override_text_color = Some(TEXT_COLOR);
            style.visuals.widgets.active.bg_fill = DESELECTED_COLOR;
            style.visuals.widgets.inactive.bg_fill = DESELECTED_COLOR;
            style.visuals.widgets.open.bg_fill = DESELECTED_COLOR;
            style.visuals.widgets.hovered.bg_fill = SELECTED_COLOR;
            ui.vertical_centered_justified(|ui| {
                #[cfg(debug_assertions)]
                {
                    if ui.button("CREDITS").clicked() {
                        action_queue.push(Action::CheatCredits);
                    }
                    if ui.button("NEXT LEVEL").clicked() {
                        action_queue.push(Action::CheatLevel);
                    }
                }

                let delivered_hats = outgoing_hats.single().input.0.get(&R::BigHats).unwrap();
                egui::Grid::new("DELIVERED grid").show(ui, |ui| {
                    ui.label(" HATS DELIVERED");
                    ui.label(&format!("{}", delivered_hats));
                    ui.end_row();
                    ui.label(" HATS ORDERED ");
                    ui.label(&format!("{}", player.required_hats));
                    ui.end_row();
                });
                ui.label(" DELIVERY DEADLINE");
                ui.label(&format!("{} SECONDS LEFT", player.delivery_dealine as i64));

                //let v = 1.0 - (player.level_time * 0.1 - player.level).fract();
                //egui::Grid::new("level grid").show(ui, |ui| {
                //    ui.label(&format!(" LEVEL {}", player.level as u32));
                //    ui.label(&format!("NEXT {:.2}", v * 10.0));
                //    ui.end_row();
                //});

                if player.selected_entity.is_some() {
                    player.item_to_place = None;
                    player.sell_mode = false;
                }

                if player.alive() {
                    ui.label("");
                    ui.label("BUILD");
                    ui_buy_button(&mut ctx, ui, "BLOBBY", Item::Blobby, &mut player);
                    ui.label("");
                    ui.label("REFINERIES");
                    ui_buy_button(&mut ctx, ui, "COPPER", Item::CopperRefinery, &mut player);
                    ui_buy_button(&mut ctx, ui, "LITHIUM", Item::LithiumRefinery, &mut player);
                    ui_buy_button(&mut ctx, ui, "GLASS", Item::GlassRefinery, &mut player);
                    ui.label("");
                    ui.label("FACTORIES");
                    ui_buy_button(
                        &mut ctx,
                        ui,
                        "LITTLE HAT",
                        Item::LittleHatFactory,
                        &mut player,
                    );
                    ui_buy_button(&mut ctx, ui, "BATTERY", Item::BatteryFactory, &mut player);
                    ui_buy_button(
                        &mut ctx,
                        ui,
                        "LIGHT BULB",
                        Item::LightbulbFactory,
                        &mut player,
                    );
                    ui_buy_button(&mut ctx, ui, "BIG HAT", Item::BigHatFactory, &mut player);

                    ui.label("");
                    if select_button(ui, "SELL", player.sell_mode).clicked() {
                        player.sell_mode = !player.sell_mode;
                        if player.sell_mode {
                            player.item_to_place = None;
                            player.selected_entity = None;
                        }
                    }
                    ui.label("");

                    ui.label(&format!("GAME SPEED {:.2}", player.time_multiplier));
                    ui.horizontal(|ui| {
                        if ui.button(" -- ").clicked() {
                            action_queue.push(Action::GameSpeedDec);
                        }
                        if ui.button(" ++ ").clicked() {
                            action_queue.push(Action::GameSpeedInc);
                        }
                        if ui.button("PAUSE").clicked() {
                            action_queue.push(Action::GamePause);
                        }
                    });
                }
                //if ui
                //    .checkbox(&mut pref.less_lights, "REDUCE LIGHTS")
                //    .changed()
                //{
                //    if pref.less_lights {
                //        pref.light_r = 0.6;
                //    } else {
                //        pref.light_r = 1.0;
                //    }
                //}

                //ui.horizontal(|ui| {
                //    if ui.button(" -- ").clicked() {
                //        pref.sfx = (pref.sfx - 0.1).max(0.0);
                //        **audio_events |= SFX_LEVEL_CHANGED;
                //    }
                //    if ui.button(" ++ ").clicked() {
                //        pref.sfx = (pref.sfx + 0.1).min(3.0);
                //        **audio_events |= SFX_LEVEL_CHANGED;
                //    }
                //    ui.label(&format!("SFX {:.1}", pref.sfx));
                //});

                ui.horizontal(|ui| {
                    if ui.button(" -- ").clicked() {
                        pref.music = (pref.music - 0.1).max(0.0);
                        **audio_events |= MUSIC_LEVEL_CHANGED;
                    }
                    if ui.button(" ++ ").clicked() {
                        pref.music = (pref.music + 0.1).min(3.0);
                        **audio_events |= MUSIC_LEVEL_CHANGED;
                    }
                    ui.label(&format!("MUSIC {:.1}", pref.music));
                });
                ui.label("");
                if ui.button("RESTART GAME").clicked() {
                    action_queue.push(Action::RestartGame);
                    game_recorder.disable_rec = false;
                    game_recorder.play = false;
                    game_recorder.actions = ActionRecording::default();
                }
                /*
                if select_button(ui, "REPLAY", game_recorder.play) {
                    action_queue.push(Action::RestartGame);
                    game_recorder.play = true;
                    game_recorder.disable_rec = true;
                    game_recorder.play_head = 0;
                }
                if ui.text_edit_singleline(&mut *rec_string).changed() {
                    if let Ok(compressed) = base64::decode(rec_string.trim()) {
                        if let Ok(bytes) = decompress_size_prepended(&compressed) {
                            if let Ok(archived) =
                                rkyv::check_archived_root::<ActionRecording>(&bytes)
                            {
                                if let Ok(deserialized) =
                                    archived.deserialize(&mut rkyv::Infallible)
                                {
                                    game_recorder.actions = deserialized;
                                }
                            }
                        }
                    }
                }
                if player_died_this_frame || ui.button("GET REPLAY STRING").clicked() {
                    let bytes = rkyv::to_bytes::<_, 1024>(&game_recorder.actions).unwrap();
                    let compressed = compress_prepend_size(&bytes);
                    let base64_bytes = base64::encode(compressed);
                    *rec_string = base64_bytes;
                }
                */
            });
        });
}

fn ui_sidebar_left(
    mut ctx: ResMut<EguiContext>,
    player: Res<PlayerState>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();
    let my_frame = egui::containers::Frame {
        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 0),
        stroke: egui::Stroke::NONE,
        ..default()
    };

    egui::SidePanel::left("left_panel")
        .frame(my_frame)
        .resizable(false)
        //.min_width(window.width() * 0.17)
        //.default_width(window.width() * 0.17)
        .max_width(window.width() * 0.17)
        .show_separator_line(false)
        .show(ctx.ctx_mut(), |ui| {
            let mut style = ui.style_mut();
            style.visuals.override_text_color = Some(TEXT_COLOR);
            style.visuals.widgets.active.bg_fill = DESELECTED_COLOR;
            style.visuals.widgets.inactive.bg_fill = DESELECTED_COLOR;
            style.visuals.widgets.open.bg_fill = DESELECTED_COLOR;
            style.visuals.widgets.hovered.bg_fill = SELECTED_COLOR;
            ui.vertical_centered_justified(|ui| {
                ui.label("RESOURCES");
                ui.separator();
                player
                    .combined_resources
                    .draw("player", ui, true, true, true);
            });
        });
}

pub fn setup_fonts(mut ctx: ResMut<EguiContext>) {
    let mut fonts = FontDefinitions::default();

    for (_text_style, mut data) in fonts.font_data.iter_mut() {
        data.tweak.scale = 1.5;
        data.font =
            std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/ShareTechMono-Regular.ttf"));
    }
    ctx.ctx_mut().set_fonts(fonts);
}

#[derive(Resource)]
pub struct Preferences {
    pub less_lights: bool,
    pub light_r: f32, //light range mult
    pub sfx: f64,
    pub music: f64,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            less_lights: false,
            light_r: 1.0,
            sfx: 1.0,
            music: 1.0,
        }
    }
}
