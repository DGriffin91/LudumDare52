use bevy::{math::*, prelude::*, utils::HashMap};

use bevy_scene_hook::{HookedSceneBundle, SceneHook};

use crate::{
    assets::ModelAssets,
    board::GameBoard,
    player::{PlayerState, Resources, R},
    schedule::TIMESTEP,
};
use int_enum::IntEnum;

#[derive(Component, Default)]
pub struct Path {
    pub path: Option<(Vec<IVec2>, u32)>,
    pub new_rand_loc_timer: f32,
}

#[repr(u8)]
#[derive(Clone, Copy, Component, PartialEq, Eq, Debug, IntEnum)]
pub enum Item {
    Blobby = 0,
    CopperRefinery = 1,
    LithiumRefinery = 2,
    GlassRefinery = 3,
    BatteryFactory = 4,
    LittleHatFactory = 5,
    BigHatFactory = 6,
    LightbulbFactory = 7,
    OutgoingHatsFactory = 8,
}

impl Item {
    pub fn cost(&self) -> Resources {
        match self {
            Item::Blobby => Resources(HashMap::from([(R::Plastic, 10), (R::LittleHats, 1)])),
            Item::CopperRefinery => Resources(HashMap::from([(R::Plastic, 30)])),
            Item::LithiumRefinery => Resources(HashMap::from([(R::Plastic, 30)])),
            Item::GlassRefinery => Resources(HashMap::from([(R::Plastic, 30), (R::Copper, 5)])),
            Item::BatteryFactory => Resources(HashMap::from([(R::Plastic, 50), (R::Copper, 5)])),
            Item::LittleHatFactory => Resources(HashMap::from([(R::Plastic, 50), (R::Copper, 5)])),
            Item::BigHatFactory => Resources(HashMap::from([(R::Plastic, 50), (R::Copper, 5)])),
            Item::LightbulbFactory => Resources(HashMap::from([(R::Plastic, 50), (R::Copper, 5)])),
            Item::OutgoingHatsFactory => {
                Resources(HashMap::from([(R::Plastic, 50), (R::Copper, 5)]))
            }
        }
    }
    pub fn name(&self) -> String {
        String::from(match self {
            Item::Blobby => "BLOBBY",
            Item::CopperRefinery => "COPPER REFINERY",
            Item::LithiumRefinery => "LITHIUM REFINERY",
            Item::GlassRefinery => "GLASS REFINERY",
            Item::BatteryFactory => "BATTERY FACTORY",
            Item::LittleHatFactory => "LITTLE HAT FACTORY",
            Item::BigHatFactory => "BIG HAT FACTORY",
            Item::LightbulbFactory => "LIGHTBULB FACTORY",
            Item::OutgoingHatsFactory => "OUTGOING HATS",
        })
    }

    pub fn recipe(&self) -> Option<(Resources, Resources)> {
        match self {
            Item::Blobby => None,
            Item::CopperRefinery => Some((
                R::Copper.recipe(),
                Resources(HashMap::from([(R::Copper, 1)])),
            )),
            Item::LithiumRefinery => Some((
                R::Lithium.recipe(),
                Resources(HashMap::from([(R::Lithium, 1)])),
            )),
            Item::GlassRefinery => {
                Some((R::Glass.recipe(), Resources(HashMap::from([(R::Glass, 1)]))))
            }
            Item::BatteryFactory => Some((
                R::Batteries.recipe(),
                Resources(HashMap::from([(R::Batteries, 1)])),
            )),
            Item::LittleHatFactory => Some((
                R::LittleHats.recipe(),
                Resources(HashMap::from([(R::LittleHats, 1)])),
            )),
            Item::BigHatFactory => Some((
                R::BigHats.recipe(),
                Resources(HashMap::from([(R::BigHats, 1)])),
            )),
            Item::LightbulbFactory => Some((
                R::Lightbulbs.recipe(),
                Resources(HashMap::from([(R::Lightbulbs, 1)])),
            )),
            Item::OutgoingHatsFactory => None,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Blobby {
    pub speed: f32,
    pub id: u8,
    pub dest: Option<IVec2>,
    //The resource pile the user has selected
    pub resource_pile: Option<Entity>,
    // The closest place that takes the remaining resources as an input
    // If there is no place this should be set to None to the blobby goes back to get more resources
    pub drop_off: Option<Entity>,
    pub going_to_pickup: bool,
}

#[derive(Component)]
pub struct PathInd;

pub(crate) fn update_blobby_paths(
    b: Res<GameBoard>,
    mut blobbies: Query<(&Transform, &mut Path, &Blobby)>,
    player: Res<PlayerState>,
) {
    if !player.alive() {
        return;
    }
    for (trans, mut blobby_path, blobby) in blobbies.iter_mut() {
        if let Some(dest) = blobby.dest {
            blobby_path.path = b.path(b.ws_vec3_to_ls(trans.translation), dest);
        }
    }
}

#[allow(dead_code)]
pub(crate) fn debug_show_blobby_path(
    b: Res<GameBoard>,
    blobbies: Query<(&Transform, &Path)>,
    mut com: Commands,
    path_ind: Query<Entity, With<PathInd>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &path_ind {
        com.entity(entity).despawn_recursive();
    }
    if let Some((_, blobby_path)) = blobbies.iter().next() {
        for path in &blobby_path.path {
            for p in &path.0 {
                com.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::UVSphere {
                        radius: 0.25,
                        ..default()
                    })),
                    material: materials.add(Color::rgb(0.0, 0.5, 0.0).into()),
                    transform: Transform::from_translation(
                        b.ls_to_ws_vec3(*p) + vec3(0.0, 0.5, 0.0),
                    ),
                    ..default()
                })
                .insert(PathInd);
            }
        }
    }
}

pub(crate) fn move_blobby_along_path(
    b: Res<GameBoard>,
    mut blobbies: Query<(&mut Transform, &mut Path, &Blobby)>,
) {
    for (mut trans, path, blobby) in blobbies.iter_mut() {
        if let Some(path) = &path.path {
            if path.0.len() > 1 {
                let p = trans.translation;
                let a = b.ls_to_ws_vec3(path.0[1]);
                let next_pos = a;
                //if !b.has_blobby[b.ls_to_idx(b.ws_vec3_to_ls(next_pos))] {
                trans.translation += (next_pos - p).normalize() * TIMESTEP * blobby.speed;
                //}
                let prev_rot = trans.rotation;
                let mut new_trans = *trans;
                new_trans.look_at(next_pos, Vec3::Y);
                trans.rotation = prev_rot.lerp(new_trans.rotation, 0.1);
            }
        }
    }
}

pub(crate) fn blobby_get_resource(
    b: Res<GameBoard>,
    mut blobbies: Query<
        (&Transform, &mut Blobby, &mut Resources),
        (Without<Pickup>, Without<Pickup>),
    >,
    mut pickups: Query<(&Transform, &mut Resources), (With<Pickup>, Without<Blobby>)>,
) {
    for (blobby_trans, mut blobby, mut blobby_resources) in &mut blobbies {
        if let Some(blob_resource_pile) = blobby.resource_pile {
            if let Ok((pickup_trans, mut pickup_resource)) = pickups.get_mut(blob_resource_pile) {
                let mut blobby_missing_res = false;
                for (k, v) in pickup_resource.0.iter() {
                    if *v > 0 {
                        if let Some(blobby_r) = blobby_resources.0.get(k) {
                            if *blobby_r == 0 {
                                blobby_missing_res = true;
                            }
                        } else {
                            blobby_missing_res = true;
                        }
                    }
                }
                blobby.going_to_pickup = false;
                if blobby_missing_res {
                    if blobby_trans.translation.distance(pickup_trans.translation) < 1.8 {
                        // Pick up ore
                        pickup_resource.take(&Resources::one(), &mut blobby_resources, false);
                        blobby.going_to_pickup = false;
                    } else {
                        let ore_pos = b.ws_vec3_to_ls(pickup_trans.translation);
                        blobby.dest = Some(ore_pos + IVec2::new(0, 1));
                        blobby.going_to_pickup = true;
                    }
                }
            }
        }
    }
}

pub(crate) fn blobby_put_resource(
    b: Res<GameBoard>,
    mut blobbies: Query<
        (&Transform, &mut Blobby, &mut Resources),
        (Without<Pickup>, Without<Pickup>),
    >,
    mut dropoffs: Query<(Entity, &Transform, &mut Dropoff), Without<Blobby>>,
) {
    for (blobby_trans, mut blobby, mut blobby_resources) in &mut blobbies {
        if blobby.resource_pile.is_some() && !blobby.going_to_pickup {
            let mut closest = 99999.0;
            let mut closest_dropoff = None;
            for (dropoff_entity, dropoff_trans, dropoff) in &dropoffs {
                let dist = blobby_trans.translation.distance(dropoff_trans.translation);
                let needs = dropoff.input.needs(&dropoff.qty, &blobby_resources);
                if dist < closest && needs {
                    closest_dropoff = Some(dropoff_entity);
                    closest = dist;
                }
            }
            if let Some(closest_dropoff) = closest_dropoff {
                if let Ok((_, dropoff_trans, mut dropoff)) = dropoffs.get_mut(closest_dropoff) {
                    if closest < 1.8 {
                        let qty = &dropoff.qty.clone();
                        blobby_resources.take(qty, &mut dropoff.input, false);
                    }
                    blobby.drop_off = Some(closest_dropoff);
                    let dropoff_pos = b.ws_vec3_to_ls(dropoff_trans.translation);
                    blobby.dest = Some(dropoff_pos + IVec2::new(0, 1));
                }
            }
        }
    }
}

pub fn receive_plastic(
    mut plastics: Query<(&mut Resources, &mut PlasticReceiver)>,
    player: Res<PlayerState>,
) {
    for (mut res, mut plastic) in &mut plastics {
        plastic.time += 1;
        if plastic.time > 200 {
            plastic.time = 0;
            let v = res.0.get_mut(&R::Plastic).unwrap();
            *v += 20 * player.level as u64;
        }
    }
}

#[derive(Component)]
pub struct Pickup;

#[derive(Component)]
pub struct Dropoff {
    pub qty: Resources,   //The max qty that this Dropoff will take
    pub input: Resources, //the input resources this dropoff has
}

#[derive(Component)]
pub struct OutputResource(pub R);

#[derive(Component)]
pub struct ResourcesAvailableToPlayer;

#[derive(Component)]
pub struct InitialPlayerResources;

#[derive(Component)]
pub struct PlasticReceiver {
    pub time: u64,
}

#[derive(Component)]
pub struct ProcessTimer {
    pub started: bool,
    pub time: u64,
    pub length: u64,
}

pub fn spawn_ore(
    com: &mut Commands,
    model_assets: &ModelAssets,
    b: &mut GameBoard,
    pos: IVec2,
    kind: R,
) {
    let trans = b.ls_to_ws_vec3(pos);
    let mut ecmds = com.spawn_empty();
    let entity = ecmds.id();
    let mut r = Resources::zero();

    if let R::Plastic = kind {
        ecmds
            .insert(ResourcesAvailableToPlayer)
            .insert(PlasticReceiver { time: 0 });
        r.0.insert(kind, 200);
    } else {
        r.0.insert(kind, 100000000);
    }

    ecmds
        .insert(HookedSceneBundle {
            scene: SceneBundle {
                scene: match kind {
                    R::CopperOre => model_assets.copper_ore.clone(),
                    R::LithiumOre => model_assets.lithium_ore.clone(),
                    R::Plastic => model_assets.plastic_delivery_warehouse.clone(),
                    R::Sand => model_assets.sand_pile.clone(),
                    _ => model_assets.copper_ore.clone(),
                },
                transform: Transform::from_translation(trans),
                ..default()
            },
            hook: SceneHook::new(move |_entity, _cmds| {}),
        })
        .insert(r)
        .insert(Pickup);

    let idx = b.ls_to_idx(pos);
    b.board[idx] = Some(entity);
}

pub fn spawn_factory(
    com: &mut Commands,
    model_assets: &ModelAssets,
    b: &mut GameBoard,
    pos: IVec2,
    kind: R,
) {
    let trans = b.ls_to_ws_vec3(pos);
    let mut ecmds = com.spawn_empty();
    let entity = ecmds.id();
    let qty = kind.recipe(); //.mult(2);
    let r = qty.as_zero();
    ecmds
        .insert(HookedSceneBundle {
            scene: SceneBundle {
                scene: match kind {
                    R::Plastic => model_assets.factory.clone(),
                    R::LittleHats => model_assets.terrarium.clone(),
                    R::BigHats => model_assets.factory.clone(),
                    R::Batteries => model_assets.factory.clone(),
                    R::Copper => model_assets.copper_refinery.clone(),
                    R::CopperOre => model_assets.copper_ore.clone(),
                    R::Lithium => model_assets.lithium_refinery.clone(),
                    R::LithiumOre => model_assets.lithium_ore.clone(),
                    R::Lightbulbs => model_assets.factory.clone(),
                    R::Glass => model_assets.lithium_refinery.clone(),
                    R::Sand => model_assets.factory.clone(),
                },
                transform: Transform::from_translation(trans),
                ..default()
            },
            hook: SceneHook::new(move |_entity, _cmds| {}),
        })
        .insert(Dropoff { qty, input: r })
        .insert(ResourcesAvailableToPlayer)
        .insert(ProcessTimer {
            started: false,
            time: 0,
            length: kind.time(),
        })
        .insert(OutputResource(kind))
        .insert(Resources(HashMap::from([(kind, 0)])))
        .insert(Sellable);

    match kind {
        R::LittleHats => (),
        _ => {
            ecmds.insert(Pickup);
        }
    }

    let idx = b.ls_to_idx(pos);
    b.board[idx] = Some(entity);
}

#[derive(Component)]
pub struct OutgoingHats;

#[derive(Component)]
pub struct Sellable;

pub fn spawn_outgoing_hats(
    com: &mut Commands,
    model_assets: &ModelAssets,
    b: &mut GameBoard,
    pos: IVec2,
) {
    let trans = b.ls_to_ws_vec3(pos);
    let mut ecmds = com.spawn_empty();
    let entity = ecmds.id();
    let qty = Resources(HashMap::from([(R::BigHats, 99999999999)]));
    let r = qty.as_zero();
    ecmds
        .insert(HookedSceneBundle {
            scene: SceneBundle {
                scene: model_assets.outgoing_hats.clone(),
                transform: Transform::from_translation(trans),
                ..default()
            },
            hook: SceneHook::new(move |_entity, _cmds| {}),
        })
        .insert(Dropoff { qty, input: r })
        .insert(OutgoingHats);
    let idx = b.ls_to_idx(pos);
    b.board[idx] = Some(entity);
}
