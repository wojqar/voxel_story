use bevy::prelude::*;

use crate::region::{RegionCoord, REGION_SIZE_VOXELS};

#[derive(Debug, Default)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl MeshData {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty() || self.positions.is_empty()
    }
}

#[inline]
fn idx(x: i32, y: i32, z: i32) -> usize {
    debug_assert!((0..REGION_SIZE_VOXELS).contains(&x));
    debug_assert!((0..REGION_SIZE_VOXELS).contains(&y));
    debug_assert!((0..REGION_SIZE_VOXELS).contains(&z));
    (x as usize)
        + (y as usize) * (REGION_SIZE_VOXELS as usize)
        + (z as usize) * (REGION_SIZE_VOXELS as usize) * (REGION_SIZE_VOXELS as usize)
}

#[derive(Clone, Copy)]
struct FaceCell {
    voxel: u16,
    back_face: bool,
}

/// Greedy meshing over a 64³ region.
///
/// `voxels` is a dense 3D array (x-major) of voxel ids for the region.
/// The mesher treats id 0 as air.
pub fn build_region_mesh(
    _region: RegionCoord,
    voxels: &[u16],
    palette: &impl Fn(u16) -> [f32; 4],
) -> MeshData {
    let n = REGION_SIZE_VOXELS;
    debug_assert_eq!(voxels.len(), (n * n * n) as usize);

    let mut out = MeshData::default();

    // 3-axis greedy meshing using a 2D mask per slice.
    for axis in 0..3 {
        let (u_axis, v_axis) = match axis {
            0 => (1, 2), // x -> (y,z)
            1 => (0, 2), // y -> (x,z)
            _ => (0, 1), // z -> (x,y)
        };

        let du = n;
        let dv = n;
        let mut mask: Vec<Option<FaceCell>> = vec![None; (du * dv) as usize];

        for d in 0..=n {
            // Build mask: faces between slice d-1 and d.
            for v in 0..dv {
                for u in 0..du {
                    let mut a = [0i32; 3];
                    let mut b = [0i32; 3];
                    a[axis as usize] = d - 1;
                    b[axis as usize] = d;
                    a[u_axis as usize] = u;
                    a[v_axis as usize] = v;
                    b[u_axis as usize] = u;
                    b[v_axis as usize] = v;

                    let av = if a[axis as usize] >= 0 {
                        voxels[idx(a[0], a[1], a[2])]
                    } else {
                        0
                    };
                    let bv = if b[axis as usize] < n {
                        voxels[idx(b[0], b[1], b[2])]
                    } else {
                        0
                    };

                    let cell = if av != 0 && bv == 0 {
                        Some(FaceCell {
                            voxel: av,
                            back_face: false, // normal +axis
                        })
                    } else if av == 0 && bv != 0 {
                        Some(FaceCell {
                            voxel: bv,
                            back_face: true, // normal -axis
                        })
                    } else {
                        None
                    };

                    mask[(u + v * du) as usize] = cell;
                }
            }

            // Greedy merge rectangles in the mask.
            let mut v = 0i32;
            while v < dv {
                let mut u = 0i32;
                while u < du {
                    let i = (u + v * du) as usize;
                    let Some(cell) = mask[i] else {
                        u += 1;
                        continue;
                    };

                    // Width.
                    let mut w = 1i32;
                    while u + w < du {
                        let j = (u + w + v * du) as usize;
                        match mask[j] {
                            Some(c) if c.voxel == cell.voxel && c.back_face == cell.back_face => w += 1,
                            _ => break,
                        }
                    }

                    // Height.
                    let mut h = 1i32;
                    'height: while v + h < dv {
                        for k in 0..w {
                            let j = (u + k + (v + h) * du) as usize;
                            match mask[j] {
                                Some(c) if c.voxel == cell.voxel && c.back_face == cell.back_face => {}
                                _ => break 'height,
                            }
                        }
                        h += 1;
                    }

                    // Emit quad.
                    emit_quad(
                        &mut out,
                        axis,
                        u_axis,
                        v_axis,
                        d,
                        u,
                        v,
                        w,
                        h,
                        cell.voxel,
                        cell.back_face,
                        &palette(cell.voxel),
                    );

                    // Clear mask.
                    for y in 0..h {
                        for x in 0..w {
                            mask[(u + x + (v + y) * du) as usize] = None;
                        }
                    }

                    u += w;
                }
                v += 1;
            }
        }
    }

    out
}

#[allow(clippy::too_many_arguments)]
fn emit_quad(
    out: &mut MeshData,
    axis: i32,
    u_axis: i32,
    v_axis: i32,
    d: i32,
    u: i32,
    v: i32,
    w: i32,
    h: i32,
    _voxel: u16,
    back_face: bool,
    color: &[f32; 4],
) {
    let mut p0 = [0f32; 3];
    let mut p1 = [0f32; 3];
    let mut p2 = [0f32; 3];
    let mut p3 = [0f32; 3];

    // Plane position along axis. If back_face, the plane is at d, otherwise at d.
    // The mask is built between d-1 and d; for faces with normal -axis (back_face),
    // the quad lies at d; for +axis it also lies at d.
    let plane = d as f32;

    let u0 = u as f32;
    let v0 = v as f32;
    let u1 = (u + w) as f32;
    let v1 = (v + h) as f32;

    p0[axis as usize] = plane;
    p1[axis as usize] = plane;
    p2[axis as usize] = plane;
    p3[axis as usize] = plane;

    p0[u_axis as usize] = u0;
    p0[v_axis as usize] = v0;
    p1[u_axis as usize] = u1;
    p1[v_axis as usize] = v0;
    p2[u_axis as usize] = u1;
    p2[v_axis as usize] = v1;
    p3[u_axis as usize] = u0;
    p3[v_axis as usize] = v1;

    let normal = match (axis, back_face) {
        (0, false) => [1.0, 0.0, 0.0],
        (0, true) => [-1.0, 0.0, 0.0],
        (1, false) => [0.0, 1.0, 0.0],
        (1, true) => [0.0, -1.0, 0.0],
        (_, false) => [0.0, 0.0, 1.0],
        (_, true) => [0.0, 0.0, -1.0],
    };

    let base = out.positions.len() as u32;
    out.positions.extend([p0, p1, p2, p3]);
    out.normals.extend([normal, normal, normal, normal]);
    out.colors.extend([*color, *color, *color, *color]);

    // Winding: ensure triangles face the intended normal across all axis permutations.
    // (u_axis,v_axis) ordering can flip winding depending on the slice plane, so we
    // compute a geometric normal and flip indices if needed.
    let tri_n = tri_normal(p0, p1, p2);
    let wants_flip = dot3(tri_n, normal) < 0.0;

    if wants_flip {
        out.indices
            .extend([base, base + 2, base + 1, base, base + 3, base + 2]);
    } else {
        out.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

#[inline]
fn tri_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let a = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let b = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

