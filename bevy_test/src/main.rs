use bevy::prelude::*;
use bevy_test::prelude::*;

fn main() {
    let (map_width, map_height, wall_data) = bevy_test::load_map_from_csv("assets/map.csv");
    let map_width_f = map_width as f32 * CELL_SIZE;
    let map_height_f = map_height as f32 * CELL_SIZE;
    let flow_grid = FlowGrid::new(map_width_f, map_height_f, CELL_SIZE, &wall_data);
    let spatial_grid = SpatialGrid::new(map_width_f, map_height_f, CELL_SIZE, &wall_data);
    let thing_catalog = ThingCatalog::new();
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<MouseState>()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(flow_grid)
        .insert_resource(spatial_grid)
        .insert_resource(thing_catalog)
        .add_plugins(UIPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(UnitPlugin)
        .add_plugins(MovementPlugin)
        .add_systems(Startup, bevy_test::setup)
        .run();
}
