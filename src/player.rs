use bevy::{math::*, prelude::*, utils::HashMap};
use bevy_egui::{
    egui::{self, Ui},
    EguiContext,
};
use bevy_mod_raycast::{Intersection, RaycastMethod, RaycastSource};

use crate::{
    action::{Action, ActionQueue},
    assets::ModelAssets,
    board::GameBoard,
    items::{
        Blobby, Dropoff, Item, OutgoingHats, OutputResource, Pickup, ProcessTimer,
        ResourcesAvailableToPlayer,
    },
    schedule::TIMESTEP,
    ui::TEXT_COLOR2,
};

pub struct GameSettings {
    pub blobby_speed: f32,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum R {
    Plastic,
    LittleHats,
    BigHats,
    Batteries,
    Copper,
    CopperOre,
    Lithium,
    LithiumOre,
    Lightbulbs,
    Glass,
    Sand,
}

#[derive(Component, Clone, Debug)]
pub struct Resources(pub HashMap<R, u64>);

impl R {
    pub fn name(&self) -> String {
        String::from(match self {
            R::Plastic => "PLASTIC",
            R::LittleHats => "LITTLE HATS",
            R::BigHats => "BIG HATS",
            R::Batteries => "BATTERIES",
            R::Copper => "COPPER",
            R::CopperOre => "COPPER ORE",
            R::Lithium => "LITHIUM",
            R::LithiumOre => "LITHIUM ORE",
            R::Lightbulbs => "LIGHT BULBS",
            R::Glass => "GLASS",
            R::Sand => "SAND",
        })
    }
    pub fn recipe(&self) -> Resources {
        match self {
            R::Plastic => Resources(HashMap::from([(R::Plastic, 1)])),
            R::LittleHats => Resources(HashMap::from([(R::Plastic, 10)])),
            R::BigHats => Resources(HashMap::from([
                (R::Plastic, 10),
                (R::Batteries, 1),
                (R::Lightbulbs, 1),
            ])),
            R::Batteries => Resources(HashMap::from([(R::Copper, 2), (R::Lithium, 2)])),
            R::Copper => Resources(HashMap::from([(R::CopperOre, 2)])),
            R::CopperOre => Resources(HashMap::from([(R::CopperOre, 1)])),
            R::Lithium => Resources(HashMap::from([(R::LithiumOre, 2)])),
            R::LithiumOre => Resources(HashMap::from([(R::LithiumOre, 1)])),
            R::Lightbulbs => Resources(HashMap::from([(R::Copper, 2), (R::Glass, 2)])),
            R::Glass => Resources(HashMap::from([(R::Sand, 3)])),
            R::Sand => Resources(HashMap::from([(R::Sand, 3)])),
        }
    }
    pub fn time(&self) -> u64 {
        match self {
            R::Plastic => 100,
            R::LittleHats => 300,
            R::BigHats => 200,
            R::Batteries => 150,
            R::Copper => 100,
            R::CopperOre => 100,
            R::Lithium => 100,
            R::LithiumOre => 100,
            R::Lightbulbs => 150,
            R::Glass => 100,
            R::Sand => 100,
        }
    }
}

fn draw_row(
    ui: &mut Ui,
    name: &str,
    val: Option<&u64>,
    show_if_zero: bool,
    show_if_none: bool,
) -> bool {
    if !show_if_none && val.is_none() {
        return false;
    }
    let val = if let Some(val) = val { *val } else { 0 };
    let show = show_if_zero || val > 0;
    if show_if_zero || val > 0 {
        ui.label(name);
        ui.label(&format!("{}", val));
        ui.end_row();
    }
    show
}

impl Resources {
    pub fn player_default() -> Self {
        let mut r = HashMap::new();
        r.insert(R::Plastic, 0);
        r.insert(R::LittleHats, 3);
        r.insert(R::BigHats, 0);
        r.insert(R::Batteries, 0);
        r.insert(R::Copper, 0);
        r.insert(R::CopperOre, 0);
        r.insert(R::Lithium, 0);
        r.insert(R::LithiumOre, 0);
        r.insert(R::Lightbulbs, 0);
        r.insert(R::Glass, 0);
        r.insert(R::Sand, 0);
        Resources(r)
    }

    pub fn zero() -> Self {
        Resources(HashMap::new())
    }

    pub fn zero_all_keys() -> Self {
        let mut r = HashMap::new();
        r.insert(R::Plastic, 0);
        r.insert(R::LittleHats, 0);
        r.insert(R::BigHats, 0);
        r.insert(R::Batteries, 0);
        r.insert(R::Copper, 0);
        r.insert(R::CopperOre, 0);
        r.insert(R::Lithium, 0);
        r.insert(R::LithiumOre, 0);
        r.insert(R::Lightbulbs, 0);
        r.insert(R::Glass, 0);
        r.insert(R::Sand, 0);
        Resources(r)
    }

    pub fn one() -> Self {
        let mut r = HashMap::new();
        r.insert(R::Plastic, 1);
        r.insert(R::LittleHats, 1);
        r.insert(R::BigHats, 1);
        r.insert(R::Batteries, 1);
        r.insert(R::Copper, 1);
        r.insert(R::CopperOre, 1);
        r.insert(R::Lithium, 1);
        r.insert(R::LithiumOre, 1);
        r.insert(R::Lightbulbs, 1);
        r.insert(R::Glass, 1);
        r.insert(R::Sand, 1);
        Resources(r)
    }

    /// Take qty from self, put into dest
    /// Returns if all requested could be taken
    pub fn take(&mut self, qty: &Resources, dest: &mut Resources, require_all: bool) -> bool {
        let mut all_requested_taken = true;

        // Check if the resources are available first
        for (k, qty_v) in qty.0.iter() {
            if let Some(self_v) = self.0.get_mut(k) {
                if *self_v < *qty_v {
                    all_requested_taken = false;
                    if require_all {
                        return all_requested_taken;
                    }
                }
            } else {
                all_requested_taken = false;
                if require_all {
                    return all_requested_taken;
                }
            }
        }

        for (k, qty_v) in qty.0.iter() {
            if let Some(self_v) = self.0.get_mut(k) {
                let take_qty = (*self_v).min(*qty_v);
                if let Some(dest_v) = dest.0.get_mut(k) {
                    *dest_v += take_qty;
                } else {
                    dest.0.insert(*k, take_qty);
                }
                *self_v = self_v.saturating_sub(take_qty);
            }
        }
        all_requested_taken
    }

    pub fn needs(&self, qty: &Resources, src: &Resources) -> bool {
        let mut needs = false;

        for (k, qty_v) in qty.0.iter() {
            if let Some(self_v) = self.0.get(k) {
                if let Some(src_v) = src.0.get(k) {
                    if *self_v < *qty_v && *src_v > 0 {
                        needs = true;
                    }
                }
            }
        }
        needs
    }

    pub fn max(&self, other: &Resources) -> Resources {
        let mut ret = Resources::zero_all_keys();

        for (key, v) in ret.0.iter_mut() {
            if let Some(n) = self.0.get(key) {
                *v = (*v).max(*n);
            }
            if let Some(n) = other.0.get(key) {
                *v = (*v).max(*n);
            }
        }

        ret
    }

    pub fn sum(&self, other: &Resources) -> Resources {
        let mut ret = Resources::zero_all_keys();

        for (key, v) in ret.0.iter_mut() {
            if let Some(n) = self.0.get(key) {
                *v += *n;
            }
            if let Some(n) = other.0.get(key) {
                *v += *n;
            }
        }

        ret
    }

    pub fn mult(&self, inp: u64) -> Resources {
        let mut ret = Resources::zero();

        for (key, v) in self.0.iter() {
            ret.0.insert(*key, *v * inp);
        }

        ret
    }

    pub fn as_zero(&self) -> Resources {
        let mut new = self.clone();
        for (_, v) in new.0.iter_mut() {
            *v = 0;
        }
        new
    }

    //pub fn max_in_other(&self, other: &Resources) -> Resources {
    //    let mut ret = Resources::zero_all_keys();
    //
    //    for (key, v) in other.0.iter_mut() {
    //        if let Some(n) = self.0.get(key) {
    //            *v = (*v).max(*n);
    //        } else {
    //            *v = (*v).max(*n);
    //        }
    //    }
    //
    //    ret
    //}

    pub fn draw(
        &self,
        id: &str,
        ui: &mut Ui,
        show_if_zero: bool,
        show_if_none: bool,
        draw_separators: bool,
    ) {
        egui::Grid::new(format!("resources grid {id}")).show(ui, |ui| {
            let a = draw_row(
                ui,
                " PLASTIC",
                self.0.get(&R::Plastic),
                show_if_zero,
                show_if_none,
            );
            let b = draw_row(
                ui,
                " SAND",
                self.0.get(&R::Sand),
                show_if_zero,
                show_if_none,
            );
            let c = draw_row(
                ui,
                " COPPER ORE",
                self.0.get(&R::CopperOre),
                show_if_zero,
                show_if_none,
            );
            let d = draw_row(
                ui,
                " LITHIUM ORE",
                self.0.get(&R::LithiumOre),
                show_if_zero,
                show_if_none,
            );

            if (a || b || c || d) && draw_separators {
                //ui.separator();
                //ui.separator();
                ui.end_row();
            }

            let a = draw_row(
                ui,
                " COPPER",
                self.0.get(&R::Copper),
                show_if_zero,
                show_if_none,
            );
            let b = draw_row(
                ui,
                " LITHIUM",
                self.0.get(&R::Lithium),
                show_if_zero,
                show_if_none,
            );
            let c = draw_row(
                ui,
                " GLASS",
                self.0.get(&R::Glass),
                show_if_zero,
                show_if_none,
            );

            if (a || b || c) && draw_separators {
                //ui.separator();
                //ui.separator();
                ui.end_row();
            }

            draw_row(
                ui,
                " BATTERIES",
                self.0.get(&R::Batteries),
                show_if_zero,
                show_if_none,
            );
            draw_row(
                ui,
                " LIGHT BULBS",
                self.0.get(&R::Lightbulbs),
                show_if_zero,
                show_if_none,
            );
            draw_row(
                ui,
                " LITTLE HATS",
                self.0.get(&R::LittleHats),
                show_if_zero,
                show_if_none,
            );
            draw_row(
                ui,
                " BIG HATS",
                self.0.get(&R::BigHats),
                show_if_zero,
                show_if_none,
            );
        });
    }
}

#[derive(Resource)]
pub struct PlayerState {
    pub combined_resources: Resources,
    pub item_to_place: Option<Item>,
    pub sell_mode: bool,
    pub level_time: f32,
    pub level: f32,
    pub time_multiplier: f64,
    pub step: u64,
    pub blobby_count: u8,
    pub selected_entity: Option<Entity>,
    pub delivery_dealine: f64,
    pub required_hats: u64,
    pub alive_set: bool,
}

pub const GAMESETTINGS: GameSettings = GameSettings { blobby_speed: 4.0 };

impl PlayerState {
    pub fn enemy_speed_boost(&self) -> f32 {
        self.level.powf(0.4) * 0.1
    }

    pub fn spawn_rate_cut(&self) -> f32 {
        self.level.powf(0.4) * 0.3
    }

    pub fn enemy_health_mult(&self) -> f32 {
        if self.level < 50.0 {
            (self.level.powf(1.32) + 1.0) / 2.0
        } else {
            (self.level.powf(1.32) + 1.0) / (2.0 - (self.level - 50.0) * 0.5).max(1.0)
        }
    }

    pub fn alive(&self) -> bool {
        self.alive_set
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState {
            combined_resources: Resources::zero_all_keys(),
            item_to_place: None,
            sell_mode: false,
            level_time: 0.0,
            level: 0.0,
            time_multiplier: 1.0,
            step: 0,
            blobby_count: 0,
            selected_entity: None,
            delivery_dealine: 50000.0,
            required_hats: 1,
            alive_set: true,
        }
    }
}

pub fn set_level(mut player: ResMut<PlayerState>) {
    if !player.alive() {
        return;
    }
    player.level_time += TIMESTEP;
    player.level = (player.level_time / 10.0).floor();
    player.step += 1;
}

pub fn setup_player(
    mut com: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    model_assets: Res<ModelAssets>,
) {
    com.spawn(PbrBundle {
        mesh: model_assets.cube_cursor.clone(),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba(0.0, 0.0, 0.0, 0.2),
            alpha_mode: AlphaMode::Blend,
            emissive: Color::rgb(1.0, 1.0, 1.0),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, -1000.0, 0.0),
        ..default()
    })
    .insert(GameCursor);
}

pub struct MyRaycastSet;

pub fn mouse_interact(
    mut egui_context: ResMut<EguiContext>,
    intersections: Query<&Intersection<MyRaycastSet>>,
    b: Res<GameBoard>,
    buttons: Res<Input<MouseButton>>,
    mut game_cursor: Query<(&mut Transform, &mut Handle<Mesh>), With<GameCursor>>,
    mut player: ResMut<PlayerState>,
    mut action_queue: ResMut<ActionQueue>,
    // TODO use action system for replays, should only need &Blobby
    mut blobbies: Query<(Entity, &Transform, &mut Blobby, &Resources), Without<GameCursor>>,
    model_assets: Res<ModelAssets>,
    pickups: Query<(Entity, &Resources), With<Pickup>>,
    dropoffs: Query<(&Dropoff, &Resources, &OutputResource), Without<OutgoingHats>>,
    outgoing_hats: Query<&Dropoff, (With<OutgoingHats>, Without<OutputResource>)>,
) {
    let mut cursor_pos = None;
    for intersection in &intersections {
        //info!(
        //    "Distance {:?}, Position {:?}",
        //    intersection.distance(),
        //    intersection.position()
        //);
        cursor_pos = intersection.position();
    }
    let cursor_pos = if let Some(cursor_pos) = cursor_pos {
        *cursor_pos
    } else {
        return;
    };

    let cur_ls_pos = b.ws_vec3_to_ls(cursor_pos);
    let cur_idx = b.ls_to_idx(cur_ls_pos);
    let cur_ls_p = b.idx_to_ls(cur_idx);
    let cur_entity = b.board[cur_idx];

    if let Some(cur_entity) = cur_entity {
        if let Ok((dropoff, _output_resource, output_kind)) = dropoffs.get(cur_entity) {
            let id = &format!("{:?}", cur_entity);
            egui::show_tooltip(
                egui_context.ctx_mut(),
                egui::Id::new(format!("dropoff_hover{}", id)),
                |ui| {
                    let mut style = ui.style_mut();
                    style.visuals.override_text_color = Some(TEXT_COLOR2);
                    ui.label(&format!("MAKES {}", output_kind.0.name()));
                    ui.label("REQUIRES");
                    output_kind
                        .0
                        .recipe()
                        .draw(&format!("qty{}", &id), ui, false, false, false);
                    ui.label("CONTAINS");
                    dropoff
                        .input
                        .draw(&format!("input{}", &id), ui, true, false, false);
                    //ui.label("OUTPUT");
                    //output_resource.draw(&format!("output{}", &id), ui, false, false, false);
                },
            );
        }
        if let Ok((_, pickup_res)) = pickups.get(cur_entity) {
            let id = &format!("{:?}", cur_entity);
            egui::show_tooltip(
                egui_context.ctx_mut(),
                egui::Id::new(format!("pickup_hover{}", id)),
                |ui| {
                    let mut style = ui.style_mut();
                    style.visuals.override_text_color = Some(TEXT_COLOR2);
                    ui.label("OUTPUT");
                    pickup_res.draw(id, ui, false, false, false);
                },
            );
        }
        if let Ok(dropoff) = outgoing_hats.get(cur_entity) {
            let id = &format!("{:?}", cur_entity);
            egui::show_tooltip(
                egui_context.ctx_mut(),
                egui::Id::new(format!("dropoff_hover{}", id)),
                |ui| {
                    let mut style = ui.style_mut();
                    style.visuals.override_text_color = Some(TEXT_COLOR2);
                    ui.label("OUTGOING BIG HATS");
                    dropoff
                        .input
                        .draw(&format!("input{}", &id), ui, true, false, false);
                    //ui.label("OUTPUT");
                    //output_resource.draw(&format!("output{}", &id), ui, false, false, false);
                },
            );
        }
    }

    let mut hovered_blobby = None;
    let mut hovered_blobby_pos = Vec3::ZERO;
    let ls_cur_pos_f = b.ws_vec3_to_ls_f(cursor_pos);

    for (entity, trans, _, _) in &blobbies {
        let ls_trans = b.ws_vec3_to_ls_f(trans.translation);
        if ls_trans.distance(ls_cur_pos_f) < 0.7 {
            hovered_blobby = Some(entity);
            hovered_blobby_pos = b.ls_f_to_ws_vec3(ls_trans - 0.5);
        }
    }

    if let Some(entity) = hovered_blobby {
        if let Ok((_, _, blobby, resources)) = blobbies.get(entity) {
            if let Some((mut trans, mut mesh)) = game_cursor.iter_mut().next() {
                trans.translation = hovered_blobby_pos + vec3(0.0, 0.0, 0.0);
                *mesh = model_assets.sphere_cursor.clone();
            }
            let id = format!("BLOBBY{}", blobby.id);
            egui::show_tooltip(
                egui_context.ctx_mut(),
                egui::Id::new(format!("blobby_hover{}", id)),
                |ui| {
                    let mut style = ui.style_mut();
                    style.visuals.override_text_color = Some(TEXT_COLOR2);
                    ui.label(&id);
                    resources.draw(&id, ui, true, false, false);
                },
            );
        }
    } else if let Some((mut trans, mut mesh)) = game_cursor.iter_mut().next() {
        let p = b.ls_to_ws_vec3(b.ws_vec3_to_ls(cursor_pos));
        trans.translation = p + vec3(0.0, -0.4, 0.0);
        *mesh = model_assets.cube_cursor.clone();
    }

    if buttons.just_pressed(MouseButton::Left) && (cursor_pos.y - 0.0).abs() < 0.1 {
        if let Some(entity) = hovered_blobby {
            if let Ok((entity, _, _blobby, _)) = blobbies.get(entity) {
                player.selected_entity = Some(entity);
                return;
            }
        }

        if let Some(cur_entity) = cur_entity {
            if let Ok((pickup_entity, _)) = pickups.get(cur_entity) {
                if let Some(entity) = player.selected_entity {
                    // TODO use action system for replays
                    if let Ok((_entity, _, mut blobby, _)) = blobbies.get_mut(entity) {
                        blobby.resource_pile = Some(pickup_entity);
                        return;
                    }
                }
            }
        }

        if let Some(entity) = player.selected_entity {
            if let Ok((_entity, _, blobby, _)) = blobbies.get(entity) {
                action_queue.push(Action::MoveBlobby(
                    cur_ls_p.x as u8,
                    cur_ls_p.y as u8,
                    blobby.id,
                ));
                return;
            }
        }

        if player.sell_mode {
            action_queue.push(Action::SellItem(cur_ls_p.x as u8, cur_ls_p.y as u8));
        } else if let Some(selected_item) = player.item_to_place {
            let x = cur_ls_p.x as u8;
            let y = cur_ls_p.y as u8;
            action_queue.push(Action::Place(x, y, selected_item as u8));
        }
    }
}

#[derive(Component)]
pub struct GameCursor;

pub fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RaycastSource<MyRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in &mut query {
        pick_source.cast_method = RaycastMethod::Screenspace(cursor_position);
    }
}

pub fn update_player_resources(
    mut player: ResMut<PlayerState>,
    query: Query<&Resources, With<ResourcesAvailableToPlayer>>,
) {
    player.combined_resources = Resources::zero_all_keys();
    for r in &query {
        player.combined_resources = player.combined_resources.sum(r);
    }
}

// TODO move elsewhere
pub fn process_factories(
    mut query: Query<(
        &mut Resources,
        &mut ProcessTimer,
        &mut Dropoff,
        &OutputResource,
    )>,
) {
    for (mut resources, mut timer, mut dropoff, output) in query.iter_mut() {
        if timer.started {
            timer.time += 1;
            if timer.time >= timer.length {
                // add one to the output and reset the timer
                let mut out = Resources::zero();
                out.0.insert(output.0, 1);
                *resources = resources.sum(&out);
                timer.started = false;
                timer.time = 0;
            }
        } else if dropoff
            .input
            .take(&output.0.recipe(), &mut Resources::zero(), true)
        {
            // the resources were available start the timer
            timer.started = true;
            timer.time = 0;
        }
    }
}

pub fn hats_objective(
    mut player: ResMut<PlayerState>,
    outgoing_hats: Query<&Dropoff, With<OutgoingHats>>,
    mut point_lights: Query<(Entity, &mut PointLight)>,
    mut spot_lights: Query<(Entity, &mut SpotLight)>,
) {
    let delivered_hats = outgoing_hats.single().input.0.get(&R::BigHats).unwrap();
    if *delivered_hats >= player.required_hats {
        player.delivery_dealine = (50000.0 - 4000.0 * *delivered_hats as f64)
            .max(1500.0 / (*delivered_hats as f64 + 10.0).log(3.0));
        player.required_hats = delivered_hats + 1;
    }

    player.delivery_dealine -= 1.0;
    if player.delivery_dealine < 0.0 {
        player.alive_set = false;
        for (_entity, mut point_light) in &mut point_lights {
            point_light.intensity = 0.0;
        }
        for (_entity, mut spot_light) in &mut spot_lights {
            spot_light.intensity = 0.0;
        }
    }
}
