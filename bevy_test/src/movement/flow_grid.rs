use crate::prelude::*;
use bevy::prelude::*;
use chunk_flow_field::map::Map;

#[derive(Resource, Debug)]
pub struct FlowGrid {
    // grid 기반의 플로우 필드 시스템을 위한 리소스
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub map: Map,
}

impl FlowGrid {
    pub fn new(map_width: f32, map_height: f32, cell_size: f32, wall: &[bool]) -> Self {
        let width = (map_width / cell_size).ceil() as usize;
        let height = (map_height / cell_size).ceil() as usize;
        Self {
            width,
            height,
            cell_size,
            map: Map::new(width, height, wall),
        }
    }

    pub fn world_to_grid(&self, position: Vec2) -> (usize, usize) {
        let x = (position.x / self.cell_size).floor() as usize;
        let y = (position.y / self.cell_size).floor() as usize;
        (x.min(self.width - 1), y.min(self.height - 1)) // 그리드 범위 내로 제한
    }

    pub fn grid_to_world(&self, grid_pos: Vec2) -> Vec2 {
        Vec2::new(
            (grid_pos.x + 0.5) * self.cell_size,
            (grid_pos.y + 0.5) * self.cell_size,
        )
    }

    // 주어진 월드 좌표에서 플로우 필드 벡터를 샘플링하는 함수 (양선형 보간)
    pub fn sample_flow_field(&self, flow_field: &FlowField, position: Vec2) -> Option<Vec2> {
        let gx = position.x / self.cell_size;
        let gy = position.y / self.cell_size;
        let grid_x = gx.floor() as usize;
        let grid_y = gy.floor() as usize;
        let grid_goal_x = (flow_field.goal.x / self.cell_size).floor() as usize;
        let grid_goal_y = (flow_field.goal.y / self.cell_size).floor() as usize;
        if grid_x == grid_goal_x && grid_y == grid_goal_y {
            return Some(Vec2::ZERO);
        }

        let tx = gx - grid_x as f32;
        let ty = gy - grid_y as f32;
        let base_x = if tx < 0.5 {
            grid_x as isize - 1
        } else {
            grid_x as isize
        }
        .max(0) as usize;
        let base_y = if ty < 0.5 {
            grid_y as isize - 1
        } else {
            grid_y as isize
        }
        .max(0) as usize;
        let tx = if tx < 0.5 { tx + 0.5 } else { tx - 0.5 }.max(0.0);
        let ty = if ty < 0.5 { ty + 0.5 } else { ty - 0.5 }.max(0.0);
        let v00 = flow_field
            .field
            .get(base_y * self.width + base_x)
            .and_then(|v| v.to_owned());
        let v10 = if (base_x + 1) >= self.width {
            None
        } else {
            flow_field
                .field
                .get(base_y * self.width + (base_x + 1))
                .and_then(|v| v.to_owned())
        };
        let v01 = flow_field
            .field
            .get((base_y + 1) * self.width + base_x)
            .and_then(|v| v.to_owned());
        let v11 = if (base_x + 1) >= self.width {
            None
        } else {
            flow_field
                .field
                .get((base_y + 1) * self.width + (base_x + 1))
                .and_then(|v| v.to_owned())
        };
        let mut total = Vec2::ZERO;
        let mut weight = 0.0;

        if let Some(v) = v00 {
            let weight_gain = (1.0 - tx) * (1.0 - ty);
            if weight_gain > 0.1 {
                total += v * weight_gain;
                weight += weight_gain;
            }
        }
        if let Some(v) = v10 {
            let weight_gain = tx * (1.0 - ty);
            if weight_gain > 0.1 {
                total += v * weight_gain;
                weight += weight_gain;
            }
        }
        if let Some(v) = v01 {
            let weight_gain = (1.0 - tx) * ty;
            if weight_gain > 0.1 {
                total += v * weight_gain;
                weight += weight_gain;
            }
        }
        if let Some(v) = v11 {
            let weight_gain = tx * ty;
            if weight_gain > 0.1 {
                total += v * weight_gain;
                weight += weight_gain;
            }
        }

        if weight < f32::EPSILON {
            return None;
        }
        Some(total / weight)
    }

    pub fn gen_flow_field(&self, target: Vec2) -> anyhow::Result<Vec<Option<Vec2>>> {
        let eps = 0.001; // 작은 값으로 0 나누기 방지
        let to_vec2 = |(x, y)| {
            if x < eps && x > -eps && y < eps && y > -eps {
                Vec2::ZERO
            } else {
                Vec2::new(x as f32, y as f32)
            }
        };
        let goal = chunk_flow_field::types::Pos {
            x: (target.x / self.cell_size).floor() as usize,
            y: (target.y / self.cell_size).floor() as usize,
        };
        let field = self.map.build_flow_field(goal)?;
        Ok(field.flow.into_iter().map(|opt| opt.map(to_vec2)).collect())
    }
}
