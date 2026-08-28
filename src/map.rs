use bevy::{math::f32, prelude::*};
use bevy_heightmap::{HeightMap, ValueFunctionHeightMap};
use chunk_flow_field::map::{
    Obstacle, Side, CELL_BLOCKED, EDGE_BOTTOM, EDGE_LEFT, EDGE_RIGHT, EDGE_TOP,
};

use crate::prelude::TERRAIN_HEIGHT_STEP;

#[derive(Resource, Debug, Clone)]
pub struct GameMap {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,

    pub landforms: Vec<Landform>,
}

#[derive(Debug, Clone, Copy)]
pub struct Landform {
    pub blocked_mask: u8,
    height: u8,
}

impl GameMap {
    pub fn from_tmx(path: impl AsRef<std::path::Path>, cell_size: f32) -> anyhow::Result<Self> {
        let mut loader = tiled::Loader::new();
        let map = loader.load_tmx_map(path)?;

        let width = map.width as usize;
        let height = map.height as usize;
        // for layer in map.layers() {
        //     println!("layer: {}", layer.name);
        //     let Some(tile_layer) = layer.as_tile_layer() else {
        //         continue;
        //     };
        //     for y in 0..map.height {
        //         for x in 0..map.width {
        //             let tile = tile_layer.get_tile(x as i32, y as i32);
        //             if let Some(tile) = tile {
        //                 print!("{} ", tile.id());
        //             } else {
        //                 print!(". ");
        //             }
        //         }
        //         println!();
        //     }
        // }
        let mut landforms = vec![
            Landform {
                blocked_mask: 0,
                height: 0,
            };
            width * height
        ];

        let tile_layer = map
            .layers()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No layers found in map"))?
            .as_tile_layer()
            .ok_or_else(|| anyhow::anyhow!("First layer is not a tile layer"))?;

        for y in 0..height {
            for x in 0..width {
                match tile_layer.get_tile(x as i32, y as i32) {
                    Some(tile) => {
                        let index = (y as usize) * width + (x as usize);
                        let height = match tile.id() {
                            0 => 0, // crevasse
                            1 => 4, // high
                            2 => 2, // low
                            3 => 3, // slope
                            6 => 4, // high
                            7 => 2, // low
                            _ => 0,
                        };
                        landforms[index] = Landform {
                            blocked_mask: if tile.id() == 0 { CELL_BLOCKED } else { 0 },
                            height,
                        };
                    }
                    None => {
                        landforms[(y as usize) * width + (x as usize)] = Landform {
                            blocked_mask: CELL_BLOCKED,
                            height: 0,
                        };
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..(width - 1) {
                let mut left = landforms[y * width + x];
                let mut right = landforms[y * width + (x + 1)];
                if left.height.abs_diff(right.height) > 1 {
                    left.blocked_mask |= EDGE_RIGHT;
                    right.blocked_mask |= EDGE_LEFT;
                    landforms[y * width + x] = left;
                    landforms[y * width + (x + 1)] = right;
                }
            }
        }
        for y in 0..(height - 1) {
            for x in 0..width {
                let mut top = landforms[y * width + x];
                let mut bottom = landforms[(y + 1) * width + x];
                if top.height.abs_diff(bottom.height) > 1 {
                    top.blocked_mask |= EDGE_BOTTOM;
                    bottom.blocked_mask |= EDGE_TOP;
                    landforms[y * width + x] = top;
                    landforms[(y + 1) * width + x] = bottom;
                }
            }
        }
        Ok(GameMap {
            width,
            height,
            cell_size,
            landforms,
        })
    }
    pub fn build_obstacle(&self) -> Obstacle {
        let mut obstacle = Obstacle::new(self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let landform = self.landforms[y * self.width + x];
                if landform.blocked_mask & CELL_BLOCKED != 0 {
                    obstacle.set_cell_blocked(x, y);
                }
                if landform.blocked_mask & EDGE_RIGHT != 0 {
                    obstacle.set_edge(x, y, Side::Right);
                }
                if landform.blocked_mask & EDGE_LEFT != 0 {
                    obstacle.set_edge(x, y, Side::Left);
                }
                if landform.blocked_mask & EDGE_TOP != 0 {
                    obstacle.set_edge(x, y, Side::Top);
                }
                if landform.blocked_mask & EDGE_BOTTOM != 0 {
                    obstacle.set_edge(x, y, Side::Bottom);
                }
            }
        }
        obstacle
    }
}

#[derive(Resource, Debug, Clone)]
pub struct TerrainHeightMap {
    pub width: usize,
    pub height: usize,
    pub sample_spacing: f32,
    pub heights: Vec<f32>,
    pub mesh: Option<Handle<Mesh>>,
}

impl TerrainHeightMap {
    pub fn from_game_map(game_map: &GameMap) -> Self {
        let width = game_map.width;
        let height = game_map.height;
        let sample_spacing = game_map.cell_size;
        let heights = game_map
            .landforms
            .iter()
            .map(|landform| landform.height as f32)
            .collect::<Vec<f32>>();
        TerrainHeightMap {
            width,
            height,
            sample_spacing,
            heights,
            mesh: None,
        }
    }

    pub fn spawn_map(
        &mut self,
        commands: &mut Commands,
        materials: &mut Assets<StandardMaterial>,
        meshes: &mut Assets<Mesh>,
    ) {
        let world_width = (self.width - 1) as f32 * self.sample_spacing;
        let world_height = (self.height - 1) as f32 * self.sample_spacing;
        let heightmap = ValueFunctionHeightMap(|p: Vec2| {
            let x = ((p.x + 0.5) * (self.width as f32 - 1.0))
                .round()
                .clamp(0.0, self.width as f32 - 1.0) as usize;
            let y = ((0.5 - p.y) * (self.height as f32 - 1.0))
                .round()
                .clamp(0.0, self.height as f32 - 1.0) as usize;
            self.heights[y * self.width + x]
        });
        let mesh =
            meshes.add(heightmap.build_mesh(UVec2::new(self.width as u32, self.height as u32)));
        self.mesh = Some(mesh.clone());
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.55, 0.3),
                perceptual_roughness: 1.0,
                ..default()
            })),
            Transform::from_translation(Vec3::new(world_width * 0.5, 0.0, world_height * 0.5))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::new(world_width, world_height, TERRAIN_HEIGHT_STEP)),
        ));
    }

    pub fn height_at(&self, pos: Vec2) -> f32 {
        let gx = (pos.x / self.sample_spacing).clamp(0.0, (self.width - 1) as f32);
        let gz = (pos.y / self.sample_spacing).clamp(0.0, (self.height - 1) as f32);
        let x0 = gx.floor() as usize;
        let z0 = gz.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let z1 = (z0 + 1).min(self.height - 1);

        let tx = gx.fract();
        let tz = gz.fract();

        let h00 = self.heights[z0 * self.width + x0];
        let h10 = self.heights[z0 * self.width + x1];
        let h01 = self.heights[z1 * self.width + x0];
        let h11 = self.heights[z1 * self.width + x1];

        let top = h00.lerp(h10, tx);
        let bottom = h01.lerp(h11, tx);

        top.lerp(bottom, tz) * TERRAIN_HEIGHT_STEP
    }

    pub fn raycast(&self, ray: &Ray3d) -> Option<Vec3> {
        if ray.direction.y >= 0.0 {
            return None; // Ray is pointing upwards, no intersection with the ground
        }

        let ground_distance = -ray.origin.y / ray.direction.y;

        if ground_distance < 0.0 {
            return None; // Intersection is behind the ray's origin
        }

        let mut near = 0.0;
        let mut far = ground_distance;

        for _ in 0..16 {
            let mid = (near + far) * 0.5;
            let point = ray.get_point(mid);

            let terrain_y = self.height_at(Vec2::new(point.x, point.z));
            if point.y > terrain_y {
                near = mid; // Move the near bound up
            } else {
                far = mid; // Move the far bound down
            }
        }

        let point = ray.get_point((near + far) * 0.5);

        let max_x = (self.width - 1) as f32 * self.sample_spacing;
        let max_z = (self.height - 1) as f32 * self.sample_spacing;

        if point.x < 0.0 || point.x > max_x || point.z < 0.0 || point.z > max_z {
            return None; // Intersection is outside the terrain bounds
        }

        let y = self.height_at(Vec2::new(point.x, point.z));
        Some(Vec3::new(point.x, y, point.z))
    }
}
