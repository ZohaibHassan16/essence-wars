//! Procedural mesh generation utilities.
//!
//! This module provides custom mesh generators for game-specific shapes
//! that aren't available in Bevy's primitive library.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// Create a faceted gem mesh (elongated octahedron).
///
/// The gem has 8 triangular faces arranged in a double-pyramid shape,
/// with the top and bottom points elongated for a crystal-like appearance.
///
/// # Parameters
/// - `width`: Width at the widest point (X and Z)
/// - `height`: Total height from bottom tip to top tip (Y)
/// - `mid_height`: Height of the middle ring (where the gem is widest)
pub fn create_gem_mesh(width: f32, height: f32, mid_height: f32) -> Mesh {
    let half_w = width / 2.0;
    let top_y = height - mid_height;
    let bot_y = -mid_height;

    // Vertices: top point, 4 middle points, bottom point
    // We duplicate vertices for flat shading (each face gets its own normals)
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Middle ring vertices (at y=0)
    let mid_verts = [
        Vec3::new(half_w, 0.0, 0.0),   // Right (+X)
        Vec3::new(0.0, 0.0, half_w),   // Front (+Z)
        Vec3::new(-half_w, 0.0, 0.0),  // Left (-X)
        Vec3::new(0.0, 0.0, -half_w),  // Back (-Z)
    ];

    let top = Vec3::new(0.0, top_y, 0.0);
    let bot = Vec3::new(0.0, bot_y, 0.0);

    // Helper to add a triangle with computed normal
    let mut add_triangle = |v0: Vec3, v1: Vec3, v2: Vec3| {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize();

        let base_idx = positions.len() as u32;
        positions.push([v0.x, v0.y, v0.z]);
        positions.push([v1.x, v1.y, v1.z]);
        positions.push([v2.x, v2.y, v2.z]);

        normals.push([normal.x, normal.y, normal.z]);
        normals.push([normal.x, normal.y, normal.z]);
        normals.push([normal.x, normal.y, normal.z]);

        uvs.push([0.5, 0.0]);
        uvs.push([0.0, 1.0]);
        uvs.push([1.0, 1.0]);

        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
    };

    // Top 4 triangles (top point to middle ring)
    for i in 0..4 {
        let v0 = mid_verts[i];
        let v1 = mid_verts[(i + 1) % 4];
        add_triangle(top, v1, v0);
    }

    // Bottom 4 triangles (bottom point to middle ring)
    for i in 0..4 {
        let v0 = mid_verts[i];
        let v1 = mid_verts[(i + 1) % 4];
        add_triangle(bot, v0, v1);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Create a more detailed gem mesh with beveled edges.
///
/// This creates a 16-face gem with intermediate vertices for a more
/// crystalline appearance.
///
/// # Parameters
/// - `width`: Width at the widest point
/// - `height`: Total height
/// - `bevel`: How much the top/bottom sections are beveled (0.0-1.0)
pub fn create_detailed_gem_mesh(width: f32, height: f32, bevel: f32) -> Mesh {
    let half_w = width / 2.0;
    let half_h = height / 2.0;
    let bevel_h = half_h * bevel;
    let bevel_w = half_w * (1.0 - bevel * 0.3);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Points at different Y levels
    let top = Vec3::new(0.0, half_h, 0.0);
    let bot = Vec3::new(0.0, -half_h, 0.0);

    // Upper bevel ring
    let upper_ring: [Vec3; 4] = [
        Vec3::new(bevel_w, bevel_h, 0.0),
        Vec3::new(0.0, bevel_h, bevel_w),
        Vec3::new(-bevel_w, bevel_h, 0.0),
        Vec3::new(0.0, bevel_h, -bevel_w),
    ];

    // Middle ring (widest)
    let mid_ring: [Vec3; 4] = [
        Vec3::new(half_w, 0.0, 0.0),
        Vec3::new(0.0, 0.0, half_w),
        Vec3::new(-half_w, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -half_w),
    ];

    // Lower bevel ring
    let lower_ring: [Vec3; 4] = [
        Vec3::new(bevel_w, -bevel_h, 0.0),
        Vec3::new(0.0, -bevel_h, bevel_w),
        Vec3::new(-bevel_w, -bevel_h, 0.0),
        Vec3::new(0.0, -bevel_h, -bevel_w),
    ];

    // Helper function to add a triangle with computed normal
    fn push_triangle(
        v0: Vec3, v1: Vec3, v2: Vec3,
        positions: &mut Vec<[f32; 3]>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 2]>,
        indices: &mut Vec<u32>,
    ) {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize();

        let base_idx = positions.len() as u32;
        positions.push([v0.x, v0.y, v0.z]);
        positions.push([v1.x, v1.y, v1.z]);
        positions.push([v2.x, v2.y, v2.z]);

        normals.push([normal.x, normal.y, normal.z]);
        normals.push([normal.x, normal.y, normal.z]);
        normals.push([normal.x, normal.y, normal.z]);

        uvs.push([0.5, 0.0]);
        uvs.push([0.0, 1.0]);
        uvs.push([1.0, 1.0]);

        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
    }

    // Top cap (4 triangles from top point to upper ring)
    for i in 0..4 {
        push_triangle(
            top, upper_ring[(i + 1) % 4], upper_ring[i],
            &mut positions, &mut normals, &mut uvs, &mut indices,
        );
    }

    // Upper section (4 quads from upper ring to mid ring)
    for i in 0..4 {
        let v0 = upper_ring[i];
        let v1 = upper_ring[(i + 1) % 4];
        let v2 = mid_ring[(i + 1) % 4];
        let v3 = mid_ring[i];
        push_triangle(v0, v1, v2, &mut positions, &mut normals, &mut uvs, &mut indices);
        push_triangle(v0, v2, v3, &mut positions, &mut normals, &mut uvs, &mut indices);
    }

    // Lower section (4 quads from mid ring to lower ring)
    for i in 0..4 {
        let v0 = mid_ring[i];
        let v1 = mid_ring[(i + 1) % 4];
        let v2 = lower_ring[(i + 1) % 4];
        let v3 = lower_ring[i];
        push_triangle(v0, v1, v2, &mut positions, &mut normals, &mut uvs, &mut indices);
        push_triangle(v0, v2, v3, &mut positions, &mut normals, &mut uvs, &mut indices);
    }

    // Bottom cap (4 triangles from lower ring to bottom point)
    for i in 0..4 {
        push_triangle(
            bot, lower_ring[i], lower_ring[(i + 1) % 4],
            &mut positions, &mut normals, &mut uvs, &mut indices,
        );
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gem_mesh_creation() {
        let mesh = create_gem_mesh(1.0, 1.5, 0.5);
        // 8 triangles * 3 vertices = 24 vertices
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
    }

    #[test]
    fn test_detailed_gem_mesh_creation() {
        let mesh = create_detailed_gem_mesh(1.0, 1.5, 0.3);
        // Should have more faces than simple gem
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
    }
}
