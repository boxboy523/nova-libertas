use bevy::prelude::*;
use bevy_hui::HuiPlugin;
use nova_libertas::{
    combat::CombatPlugin, constants::CELL_SIZE, debug::DebugPlugin, input::{InputPlugin, MouseState}, map::{GameMap, TerrainHeightMap}, movement::{MovementPlugin, flow_grid::FlowGrid}, player::PlayerPlugin, thing::ThingCatalog, ui::UIPlugin, unit::{UnitPlugin, spatial_grid::SpatialGrid}, visual::{SpritePlugin, team_color::TeamColorMaterial}, world3d::World3DPlugin
};

fn main() {
    //let (map_width, map_height, wall_data) = bevy_test::load_map_from_csv("assets/map.csv");
    let game_map = GameMap::from_tmx("assets/map.tmx", CELL_SIZE).expect("Failed to load map");
    let height_map = TerrainHeightMap::from_game_map(&game_map);
    let flow_grid = FlowGrid::new(&game_map);
    let spatial_grid = SpatialGrid::new(&game_map);
    let thing_catalog = ThingCatalog::new();
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(HuiPlugin)
        .init_resource::<MouseState>()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(game_map)
        .insert_resource(flow_grid)
        .insert_resource(spatial_grid)
        .insert_resource(thing_catalog)
        .insert_resource(height_map)
        .add_plugins(SpritePlugin)
        .add_plugins(MaterialPlugin::<TeamColorMaterial>::default())
        .add_plugins(UIPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(UnitPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(World3DPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(PostStartup, nova_libertas::setup)
        .run();
}
