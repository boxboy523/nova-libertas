use bevy::prelude::*;

use crate::map::TerrainHeightMap;

/// Tests a circle against an axis-aligned rectangle, including touching boundaries.
/// `radius` must be nonnegative and `rect_min` must not exceed `rect_max`.
pub fn circle_rect_overlap(center: Vec2, radius: f32, rect_min: Vec2, rect_max: Vec2) -> bool {
    let closest = center.clamp(rect_min, rect_max);
    center.distance_squared(closest) <= radius * radius
}

/// Returns the first circle-boundary intersection along the segment from
/// `line_end` (t = 0) to `line_start` (t = 1).
/// Returns None for a near-zero-length segment or no boundary intersection;
/// a segment wholly inside the circle also returns None.
pub fn circle_line_overlap(radius: f32, center: Vec2, line_start: Vec2, line_end: Vec2) -> Option<f32> {
    let delta = line_start - line_end;
    let sigma = 0.00001;
    let a = delta.length_squared();
    if a < sigma {
        return None;
    }
    let to_center = line_end - center;
    let b_half = to_center.x * delta.x + to_center.y * delta.y;
    let c = to_center.length_squared() - radius.powi(2);
    let det = b_half.powi(2) - a * c;
    if det < 0.0 {
        return None;
    }
    let t1 = (-b_half + det.sqrt()) / a;
    let t2 = (-b_half - det.sqrt()) / a;
    let t1_valid = t1 >= 0.0 && t1 <= 1.0;
    let t2_valid = t2 >= 0.0 && t2 <= 1.0;
    if t1_valid && t2_valid {
        return Some(t1.min(t2));
    } else if t1_valid {
        return Some(t1);
    } else if t2_valid {
        return Some(t2);
    } else {
        return None;
    }
}

/// Snaps a footprint to the grid; odd-sized footprints center on the cursor cell.
/// `cell_size` must be positive.
pub fn snapped_origin(position: Vec2, size: UVec2, cell_size: f32) -> IVec2 {
    (position / cell_size).floor().as_ivec2() - (size / 2).as_ivec2()
}

/// Returns the world-space center of a grid footprint.
pub fn footprint_center(origin: IVec2, size: UVec2, cell_size: f32) -> Vec2 {
    (origin.as_vec2() + size.as_vec2() * 0.5) * cell_size
}

/// Projects a viewport position onto the terrain, retaining its world-space height.
/// The caller supplies the camera pose and handles cursor availability and UI capture.
pub fn screen_to_world(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    height_map: &TerrainHeightMap,
    screen_position: Vec2,
) -> Option<Vec3> {
    let ray = camera
        .viewport_to_world(camera_transform, screen_position)
        .ok()?;
    height_map.raycast(&ray)
}

/// Projects a viewport position onto the terrain's simulation x/z plane.
pub fn screen_to_ground(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    height_map: &TerrainHeightMap,
    screen_position: Vec2,
) -> Option<Vec2> {
    let hit = screen_to_world(camera, camera_transform, height_map, screen_position)?;
    Some(Vec2::new(hit.x, hit.z))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_line_overlap_preserves_parameter_direction_and_boundary_semantics() {
        let center = Vec2::ZERO;
        assert_eq!(circle_line_overlap(1.0, center, Vec2::new(2.0, 0.0), Vec2::new(-2.0, 0.0)), Some(0.25));
        assert_eq!(circle_line_overlap(1.0, center, Vec2::new(2.0, 0.0), center), Some(0.5));
        assert_eq!(circle_line_overlap(1.0, center, Vec2::new(2.0, 1.0), Vec2::new(-2.0, 1.0)), Some(0.5));
        assert_eq!(circle_line_overlap(1.0, center, Vec2::new(2.0, 2.0), Vec2::new(-2.0, 2.0)), None);
        assert_eq!(circle_line_overlap(1.0, center, center, center), None);
        assert_eq!(circle_line_overlap(1.0, center, Vec2::new(0.5, 0.0), center), None);
    }

    #[test]
    fn footprint_coordinates_handle_odd_even_and_negative_origins() {
        assert_eq!(snapped_origin(Vec2::new(49.0, 69.0), UVec2::splat(3), 20.0), IVec2::new(1, 2));
        assert_eq!(footprint_center(IVec2::new(1, 2), UVec2::splat(3), 20.0), Vec2::new(50.0, 70.0));
        assert_eq!(snapped_origin(Vec2::new(49.0, 69.0), UVec2::splat(2), 20.0), IVec2::new(1, 2));
        assert_eq!(footprint_center(IVec2::new(1, 2), UVec2::splat(2), 20.0), Vec2::new(40.0, 60.0));
        assert_eq!(snapped_origin(Vec2::splat(-1.0), UVec2::splat(3), 20.0), IVec2::splat(-2));
    }

    #[test]
    fn circle_rect_overlap_handles_interior_edges_corners_and_contact() {
        let min = Vec2::ZERO;
        let max = Vec2::splat(10.0);
        for (center, radius, expected) in [
            (Vec2::splat(5.0), 0.0, true),
            (Vec2::new(-2.0, 5.0), 2.0, true),
            (Vec2::new(-2.1, 5.0), 2.0, false),
            (Vec2::new(13.0, 14.0), 5.0, true),
            (Vec2::new(13.0, 14.0), 4.9, false),
            (Vec2::new(10.0, 5.0), 0.0, true),
        ] {
            assert_eq!(circle_rect_overlap(center, radius, min, max), expected);
        }
    }

    #[test]
    fn screen_projection_preserves_terrain_hit_and_ground_axes() {
        use bevy::camera::RenderTargetInfo;

        let mut camera = Camera::default();
        camera.computed.target_info = Some(RenderTargetInfo {
            physical_size: UVec2::new(800, 600),
            scale_factor: 1.0,
        });
        camera.computed.clip_from_view = Mat4::perspective_infinite_reverse_rh(
            std::f32::consts::FRAC_PI_4, 800.0 / 600.0, 0.1,
        );
        let transform = GlobalTransform::from(
            Transform::from_xyz(30.0, 100.0, 30.0)
                .looking_at(Vec3::new(30.0, 0.0, 30.0), Vec3::NEG_Z),
        );
        let heights = TerrainHeightMap {
            width: 4,
            height: 4,
            sample_spacing: 20.0,
            heights: vec![1.0; 16],
            mesh: None,
        };
        let cursor = Vec2::new(400.0, 300.0);
        let ray = camera.viewport_to_world(&transform, cursor).unwrap();
        let expected = heights.raycast(&ray).unwrap();
        assert_eq!(screen_to_world(&camera, &transform, &heights, cursor), Some(expected));
        assert_eq!(screen_to_ground(&camera, &transform, &heights, cursor),
            Some(Vec2::new(expected.x, expected.z)));

        let outside = GlobalTransform::from(
            Transform::from_xyz(300.0, 100.0, 300.0)
                .looking_at(Vec3::new(300.0, 0.0, 300.0), Vec3::NEG_Z),
        );
        assert!(screen_to_world(&camera, &outside, &heights, cursor).is_none());
        assert!(screen_to_ground(&camera, &outside, &heights, cursor).is_none());
        assert!(screen_to_world(&Camera::default(), &transform, &heights, cursor).is_none());
    }
}
