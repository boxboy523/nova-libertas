use crate::prelude::*;
use bevy::prelude::*;
use std::collections::HashSet;

// 명령 처리 시스템: 명령을 받은 유닛들을 FlowField를 따라 방향을 업데이트하는 시스템
pub fn flow_movement_system(
    mut query: Query<(&Position, &mut UnitMovement, &mut Moving)>,
    query_fields: Query<&FlowField>,
    flow_grid: Res<FlowGrid>,
) {
    let near_target_margin_squared = NEAR_TARGET_MARGIN * NEAR_TARGET_MARGIN;
    query
        .iter_mut()
        .for_each(|(position, mut movement, mut moving)| {
            let flow_field = if let Ok(field) = query_fields.get(moving.field) {
                field
            } else {
                return; // FlowField를 찾을 수 없으면 건너뜀
            };
            moving.dist_target_sq = position.distance_squared(flow_field.goal);
            movement.preferred_dir = if moving.dist_target_sq < near_target_margin_squared {
                // 목표 지점 근처에 있으면 직선 이동
                (flow_field.goal - **position).normalize_or_zero()
            } else if let Some(dir) = // 플로우 필드에서 방향 벡터를 샘플링
                flow_grid.sample_flow_field(flow_field, **position)
            {
                dir
            } else {
                Vec2::ZERO
            };
        });
}

pub fn remove_empty_orders_system(
    mut commands: Commands,
    query: Query<Entity, With<FlowField>>,
    query_moving: Query<&Moving>,
) {
    let field_using = query_moving
        .iter()
        .map(|moving| moving.field)
        .collect::<HashSet<_>>();
    query.iter().for_each(|entity| {
        if !field_using.contains(&entity) {
            commands.entity(entity).despawn();
        }
    });
}

pub fn update_flow_field_system(
    mut query_target: Query<(&mut FlowField, &FieldFollowTarget)>,
    query_position: Query<&Position>,
    grid: Res<FlowGrid>,
) {
    query_target
        .par_iter_mut()
        .for_each(|(mut flow_field, follow_target)| {
            if let Ok(target_position) = query_position.get(follow_target.0) {
                if **target_position != flow_field.goal {
                    let last_grid_pos = grid.world_to_grid(flow_field.goal);
                    let new_grid_pos = grid.world_to_grid(**target_position);
                    flow_field.goal = **target_position;
                    if last_grid_pos != new_grid_pos {
                        flow_field.field = grid
                            .gen_flow_field(flow_field.goal)
                            .unwrap_or_else(|_| vec![None; grid.width * grid.height]);
                    }
                }
            }
        });
}

pub fn flow_field_added_system(
    mut query: Query<&mut FlowField, Added<FlowField>>,
    grid: Res<FlowGrid>,
) {
    query.iter_mut().for_each(|mut flow_field| {
        flow_field.field = grid
            .gen_flow_field(flow_field.goal)
            .unwrap_or_else(|_| vec![None; grid.width * grid.height]);
    });
}
