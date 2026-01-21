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

/// Create a gem mesh with a flat front face for displaying card art.
///
/// This creates a crystal-like shape with a prominent front face that has proper
/// UV mapping (0-1 range) for the card texture. The parallax effect will work
/// on this front face.
///
/// # Parameters
/// - `width`: Width of the front face (X axis)
/// - `height`: Height of the front face (Y axis)
/// - `depth`: Depth of the gem from front to back (Z axis)
/// - `bevel`: Bevel amount for the crystal edges (0.0-0.5)
pub fn create_card_gem_mesh(width: f32, height: f32, depth: f32, bevel: f32) -> Mesh {
    let half_w = width / 2.0;
    let half_h = height / 2.0;
    let front_z = depth / 2.0;
    let back_z = -depth / 2.0;

    // Bevel offsets
    let bevel_offset_w = half_w * bevel;
    let bevel_offset_h = half_h * bevel;
    let bevel_depth = depth * bevel * 0.5;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // ========== FRONT FACE (main card display area) ==========
    // Front face corners (slightly inset from full size for bevel)
    let front_inner_corners = [
        Vec3::new(-half_w + bevel_offset_w, -half_h + bevel_offset_h, front_z - bevel_depth),  // bottom-left
        Vec3::new(half_w - bevel_offset_w, -half_h + bevel_offset_h, front_z - bevel_depth),   // bottom-right
        Vec3::new(half_w - bevel_offset_w, half_h - bevel_offset_h, front_z - bevel_depth),    // top-right
        Vec3::new(-half_w + bevel_offset_w, half_h - bevel_offset_h, front_z - bevel_depth),   // top-left
    ];

    // Front edge vertices (on the very front plane)
    let front_edge = [
        Vec3::new(-half_w, -half_h, front_z),  // bottom-left
        Vec3::new(half_w, -half_h, front_z),   // bottom-right
        Vec3::new(half_w, half_h, front_z),    // top-right
        Vec3::new(-half_w, half_h, front_z),   // top-left
    ];

    let back_point = Vec3::new(0.0, 0.0, back_z);

    // Main front quad with proper UV mapping (0-1 range)
    add_quad_to_mesh(
        front_inner_corners[0], front_inner_corners[1],
        front_inner_corners[2], front_inner_corners[3],
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );

    // Small edge UV values (bevels and sides don't display card art)
    let edge_uv = [[0.0, 0.5], [0.5, 0.5], [0.5, 0.0], [0.0, 0.0]];

    // ========== FRONT BEVEL EDGES ==========
    // Bottom bevel
    add_quad_to_mesh(
        front_edge[0], front_edge[1], front_inner_corners[1], front_inner_corners[0],
        edge_uv[0], edge_uv[1], edge_uv[2], edge_uv[3],
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );

    // Right bevel
    add_quad_to_mesh(
        front_edge[1], front_edge[2], front_inner_corners[2], front_inner_corners[1],
        edge_uv[0], edge_uv[1], edge_uv[2], edge_uv[3],
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );

    // Top bevel
    add_quad_to_mesh(
        front_edge[2], front_edge[3], front_inner_corners[3], front_inner_corners[2],
        edge_uv[0], edge_uv[1], edge_uv[2], edge_uv[3],
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );

    // Left bevel
    add_quad_to_mesh(
        front_edge[3], front_edge[0], front_inner_corners[0], front_inner_corners[3],
        edge_uv[0], edge_uv[1], edge_uv[2], edge_uv[3],
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );

    // ========== SIDE FACES ==========
    let tri_uv = [[0.0, 0.5], [0.5, 0.0], [1.0, 0.5]];

    // Bottom side
    add_triangle_to_mesh(
        front_edge[0], back_point, front_edge[1],
        tri_uv[0], tri_uv[1], tri_uv[2],
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );

    // Right side
    add_triangle_to_mesh(
        front_edge[1], back_point, front_edge[2],
        tri_uv[0], tri_uv[1], tri_uv[2],
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );

    // Top side
    add_triangle_to_mesh(
        front_edge[2], back_point, front_edge[3],
        tri_uv[0], tri_uv[1], tri_uv[2],
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );

    // Left side
    add_triangle_to_mesh(
        front_edge[3], back_point, front_edge[0],
        tri_uv[0], tri_uv[1], tri_uv[2],
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Helper to add a quad (two triangles) to mesh buffers.
#[allow(clippy::too_many_arguments)]
fn add_quad_to_mesh(
    v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3,
    uv0: [f32; 2], uv1: [f32; 2], uv2: [f32; 2], uv3: [f32; 2],
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    let edge1 = v1 - v0;
    let edge2 = v3 - v0;
    let normal = edge1.cross(edge2).normalize();
    let n = [normal.x, normal.y, normal.z];

    let base_idx = positions.len() as u32;

    // First triangle (v0, v1, v2)
    positions.push([v0.x, v0.y, v0.z]);
    positions.push([v1.x, v1.y, v1.z]);
    positions.push([v2.x, v2.y, v2.z]);
    normals.push(n);
    normals.push(n);
    normals.push(n);
    uvs.push(uv0);
    uvs.push(uv1);
    uvs.push(uv2);
    indices.push(base_idx);
    indices.push(base_idx + 1);
    indices.push(base_idx + 2);

    // Second triangle (v0, v2, v3)
    positions.push([v0.x, v0.y, v0.z]);
    positions.push([v2.x, v2.y, v2.z]);
    positions.push([v3.x, v3.y, v3.z]);
    normals.push(n);
    normals.push(n);
    normals.push(n);
    uvs.push(uv0);
    uvs.push(uv2);
    uvs.push(uv3);
    indices.push(base_idx + 3);
    indices.push(base_idx + 4);
    indices.push(base_idx + 5);
}

/// Helper to add a triangle to mesh buffers.
fn add_triangle_to_mesh(
    v0: Vec3, v1: Vec3, v2: Vec3,
    uv0: [f32; 2], uv1: [f32; 2], uv2: [f32; 2],
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let normal = edge1.cross(edge2).normalize();
    let n = [normal.x, normal.y, normal.z];

    let base_idx = positions.len() as u32;
    positions.push([v0.x, v0.y, v0.z]);
    positions.push([v1.x, v1.y, v1.z]);
    positions.push([v2.x, v2.y, v2.z]);
    normals.push(n);
    normals.push(n);
    normals.push(n);
    uvs.push(uv0);
    uvs.push(uv1);
    uvs.push(uv2);
    indices.push(base_idx);
    indices.push(base_idx + 1);
    indices.push(base_idx + 2);
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

    #[test]
    fn test_card_gem_mesh_creation() {
        let mesh = create_card_gem_mesh(0.8, 1.0, 0.4, 0.1);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    }
}
