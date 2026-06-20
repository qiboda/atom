// Pass 2: 边交叉点查找
// 对 (vc+2)³ 个体素（含双边 shell），检查 12 条边是否穿过 isosurface

struct VoxelEdgeCrossPoint {
    position: vec3<f32>,
    vpad0: u32,
    normal: vec3<f32>,
    vpad1: u32,
}

struct TerrainChunkInfo {
    chunk_min: vec3<f32>,
    pad0: u32,
    voxel_size: f32,
    voxel_count: u32,
    terrain_size: f32,
    seed: u32,
    pad1: vec2<u32>,
    pad2: vec2<u32>,
}

@group(0) @binding(0) var<uniform> chunk_info: TerrainChunkInfo;
@group(0) @binding(1) var<storage, read_write> density: array<f32>;
@group(0) @binding(2) var<storage, read_write> cross_points: array<u32>;

// ── density query ──

fn grid_idx(gx: u32, gy: u32, gz: u32) -> u32 {
    let dn = chunk_info.voxel_count + 3u; // vc+3 (双边 shell)
    return gx + gy * dn + gz * dn * dn;
}

fn read_density(idx: u32) -> f32 {
    return density[idx];
}

fn grid_pos(idx: u32) -> vec3<f32> {
    let dn = chunk_info.voxel_count + 3u;
    let z = idx / (dn * dn);
    let r = idx % (dn * dn);
    let y = r / dn;
    let x = r % dn;
    return chunk_info.chunk_min + (vec3<f32>(f32(x), f32(y), f32(z)) - 1.0) * chunk_info.voxel_size;
}

// ── edge definitions ──

const EDGE_DIRS: array<u32, 12> = array<u32, 12>(0u, 0u, 0u, 0u,  1u, 1u, 1u, 1u,  2u, 2u, 2u, 2u);
const EDGE_CORNERS: array<vec3<u32>, 12> = array<vec3<u32>, 12>(
    vec3(0u, 0u, 0u), vec3(0u, 0u, 1u), vec3(0u, 1u, 0u), vec3(0u, 1u, 1u),
    vec3(0u, 0u, 0u), vec3(1u, 0u, 0u), vec3(0u, 0u, 1u), vec3(1u, 0u, 1u),
    vec3(0u, 0u, 0u), vec3(1u, 0u, 0u), vec3(0u, 1u, 0u), vec3(1u, 1u, 0u),
);

fn write_cross_point(edge_id: u32, pos: vec3<f32>, normal: vec3<f32>, flip: bool) {
    let base = edge_id * 8u;
    cross_points[base + 0u] = bitcast<u32>(pos.x);
    cross_points[base + 1u] = bitcast<u32>(pos.y);
    cross_points[base + 2u] = bitcast<u32>(pos.z);
    cross_points[base + 3u] = bitcast<u32>(f32(flip)); // flip flag
    cross_points[base + 4u] = bitcast<u32>(normal.x);
    cross_points[base + 5u] = bitcast<u32>(normal.y);
    cross_points[base + 6u] = bitcast<u32>(normal.z);
    cross_points[base + 7u] = 0u;
}

fn estimate_normal(p: vec3<f32>) -> vec3<f32> {
    let h = chunk_info.voxel_size * 0.5;
    let dx = read_density(grid_idx_from_world(p + vec3(h, 0.0, 0.0)))
           - read_density(grid_idx_from_world(p - vec3(h, 0.0, 0.0)));
    let dy = read_density(grid_idx_from_world(p + vec3(0.0, h, 0.0)))
           - read_density(grid_idx_from_world(p - vec3(0.0, h, 0.0)));
    let dz = read_density(grid_idx_from_world(p + vec3(0.0, 0.0, h)))
           - read_density(grid_idx_from_world(p - vec3(0.0, 0.0, h)));
    return normalize(vec3(dx, dy, dz));
}

fn grid_idx_from_world(p: vec3<f32>) -> u32 {
    let rel = (p - chunk_info.chunk_min) / chunk_info.voxel_size + 1.0; // +1 偏移
    let max_grid = chunk_info.voxel_count + 2u;
    let gx = clamp(u32(round(rel.x)), 0u, max_grid);
    let gy = clamp(u32(round(rel.y)), 0u, max_grid);
    let gz = clamp(u32(round(rel.z)), 0u, max_grid);
    return grid_idx(gx, gy, gz);
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let vc = chunk_info.voxel_count;
    let vv = vc + 2u; // 双边 shell 的 voxel 数
    if gid.x >= vv || gid.y >= vv || gid.z >= vv { return; }

    for (var e = 0u; e < 12u; e++) {
        let axis = EDGE_DIRS[e];
        let corner = EDGE_CORNERS[e];

        let c0_x = gid.x + corner.x;
        let c0_y = gid.y + corner.y;
        let c0_z = gid.z + corner.z;
        var c1_x: u32 = c0_x;
        var c1_y: u32 = c0_y;
        var c1_z: u32 = c0_z;
        if axis == 0u { c1_x += 1u; }
        else if axis == 1u { c1_y += 1u; }
        else { c1_z += 1u; }

        if c1_x > vc + 2u || c1_y > vc + 2u || c1_z > vc + 2u { continue; }

        let d0 = read_density(grid_idx(c0_x, c0_y, c0_z));
        let d1 = read_density(grid_idx(c1_x, c1_y, c1_z));
        if (d0 > 0.0) == (d1 > 0.0) { continue; }

        let p0 = grid_pos(grid_idx(c0_x, c0_y, c0_z));
        let p1 = grid_pos(grid_idx(c1_x, c1_y, c1_z));
        var lo = p0;
        var hi = p1;
        var dlo = d0;

        for (var iter = 0u; iter < 8u; iter++) {
            let mid = (lo + hi) * 0.5;
            let dmid = read_density(grid_idx_from_world(mid));
            if (dmid > 0.0) == (dlo > 0.0) {
                lo = mid;
                dlo = dmid;
            } else {
                hi = mid;
            }
        }
        let cross_pos = (lo + hi) * 0.5;
        let normal = estimate_normal(cross_pos);

        let edge_id = (gid.x + gid.y * vv + gid.z * vv * vv) * 12u + e;
        write_cross_point(edge_id, cross_pos, normal, d1 < 0.0);
    }
}
