use crate::ecs::prelude::*;
use godot::prelude::*;

pub fn gen_integration_field(
    target: (usize, usize),
    unit_positions: &[(i32, i32)],
    grid: &FlowGrid,
    costs: &[u8],
) -> Vec<u32> {
    let mut field = vec![u32::MAX; grid.width * grid.height];
    let mut must_visit = vec![false; grid.width * grid.height];
    let mut must_visit_count = 0;
    let mut queue = std::collections::VecDeque::new();

    for &(x, y) in unit_positions {
        for (dx, dy) in &[
            (0, 1),
            (1, 0),
            (0, -1),
            (-1, 0),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ] {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < grid.width as i32 && ny >= 0 && ny < grid.height as i32 {
                let idx = (ny as usize) * grid.width + (nx as usize);
                if !must_visit[idx] {
                    must_visit[idx] = true;
                    must_visit_count += 1;
                }
            }
        }
    }
    let taregt_idx = target.1 * grid.width + target.0;
    field[taregt_idx] = 0;
    queue.push_back(target);

    let neighbors = [
        (0, 1, 12),
        (1, 0, 12),
        (0, -1, 12),
        (-1, 0, 12),
        (1, 1, 17),
        (1, -1, 17),
        (-1, 1, 17),
        (-1, -1, 17),
    ];

    while let Some((x, y)) = queue.pop_front() {
        let current_idx = y * grid.width + x;
        let current_cost = field[current_idx];

        for (dx, dy, weight) in neighbors.iter() {
            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if nx >= 0 && nx < grid.width as isize && ny >= 0 && ny < grid.height as isize {
                let next_idx = (ny as usize) * grid.width + (nx as usize);
                let terrain_cost = costs[next_idx] as u32;

                if terrain_cost < 255 {
                    let new_cost = current_cost + weight * terrain_cost;
                    if new_cost < field[next_idx] {
                        field[next_idx] = new_cost;
                        queue.push_back((nx as usize, ny as usize));
                    }
                }
            }
        }

        if must_visit[current_idx] {
            must_visit[current_idx] = false;
            must_visit_count -= 1;
            if must_visit_count == 0 {
                return field;
            }
        }
    }
    field
}

pub fn gen_flow_field(integration_field: &[u32], grid: &FlowGrid) -> Vec<Vector2> {
    let mut flow_field = vec![Vector2::ZERO; grid.width * grid.height];

    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y * grid.width + x;

            let current_cost = integration_field[idx];
            if current_cost == u32::MAX {
                continue; // 접근 불가능한 셀
            }

            let mut best_dir = Vector2::ZERO;
            let mut best_cost = current_cost;

            for (dx, dy) in &[
                (0, 1),
                (1, 0),
                (0, -1),
                (-1, 0),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ] {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0 && nx < grid.width as isize && ny >= 0 && ny < grid.height as isize {
                    let next_idx = (ny as usize) * grid.width + (nx as usize);
                    let next_cost = integration_field[next_idx];

                    if next_cost < best_cost {
                        best_cost = next_cost;
                        best_dir = Vector2::new(*dx as f32, *dy as f32).normalized();
                    }
                }
            }
            flow_field[idx] = best_dir;
        }
    }
    flow_field
}
