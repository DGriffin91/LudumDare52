use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

#[derive(Resource, AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/ShareTechMono-Regular.ttf")]
    pub mono_medium: Handle<Font>,
}

#[derive(Resource, AssetCollection)]
pub struct ModelAssets {
    // --- Units ---
    #[asset(path = "models/units/Blobby_Guy.glb#Scene0")]
    pub blobby_guy: Handle<Scene>,
    #[asset(path = "models/units/Factory.glb#Scene0")]
    pub factory: Handle<Scene>,
    #[asset(path = "models/units/Lithium_Refinery.glb#Scene0")]
    pub lithium_refinery: Handle<Scene>,
    #[asset(path = "models/units/Copper_Refinery.glb#Scene0")]
    pub copper_refinery: Handle<Scene>,

    // --- Ore ---
    #[asset(path = "models/units/Copper_Ore.glb#Scene0")]
    pub copper_ore: Handle<Scene>,
    #[asset(path = "models/units/Lithium_Ore.glb#Scene0")]
    pub lithium_ore: Handle<Scene>,
    #[asset(path = "models/units/Sand_Pile.glb#Scene0")]
    pub sand_pile: Handle<Scene>,
    #[asset(path = "models/units/Plastic_Delivery_Warehouse.glb#Scene0")]
    pub plastic_delivery_warehouse: Handle<Scene>,
    #[asset(path = "models/units/Outgoing_Hats.glb#Scene0")]
    pub outgoing_hats: Handle<Scene>,
    #[asset(path = "models/units/Terrarium.glb#Scene0")]
    pub terrarium: Handle<Scene>,

    // --- Misc ---
    #[asset(path = "models/misc/Game_Board.glb#Scene0")]
    pub board: Handle<Scene>,

    #[asset(path = "models/misc/cube_cursor.glb#Mesh0/Primitive0")]
    pub cube_cursor: Handle<Mesh>,
    #[asset(path = "models/misc/sphere_cursor.glb#Mesh0/Primitive0")]
    pub sphere_cursor: Handle<Mesh>,
}

#[derive(Resource, AssetCollection)]
pub struct AudioAssets {
    #[asset(path = "audio/units/laser1.flac")]
    pub laser1: Handle<AudioSource>,
    #[asset(path = "audio/units/laser2.flac")]
    pub laser2: Handle<AudioSource>,

    #[asset(path = "audio/units/con_laser.flac")]
    pub con_laser: Handle<AudioSource>,

    #[asset(path = "audio/music/music.ogg")]
    pub music: Handle<AudioSource>,
}

#[derive(Component)]
pub struct LightFixed;

pub fn fix_material_colors(
    mut com: Commands,
    mut events: EventReader<AssetEvent<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut point_lights: Query<(Entity, &mut PointLight), Without<LightFixed>>,
    mut spot_lights: Query<(Entity, &mut SpotLight), Without<LightFixed>>,
) {
    for event in events.iter() {
        //https://github.com/bevyengine/bevy/pull/6828
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut mat) = materials.get_mut(handle) {
                    let c: Vec4 = mat.base_color.into();
                    mat.base_color = Color::rgba_linear(c.x, c.y, c.z, c.w);
                    let c: Vec4 = mat.emissive.into();
                    mat.emissive = Color::rgba_linear(c.x, c.y, c.z, c.w);
                }
            }
            AssetEvent::Modified { .. } => (),
            AssetEvent::Removed { .. } => (),
        }
    }
    // Just to closer match blender, idk why it's different
    for (entity, mut point_light) in &mut point_lights {
        point_light.intensity *= 0.05;
        com.entity(entity).insert(LightFixed);
    }
    for (entity, mut spot_light) in &mut spot_lights {
        spot_light.intensity *= 0.05;
        com.entity(entity).insert(LightFixed);
    }
}
