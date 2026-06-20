// Pass 2: 边交叉点查找
// 对 voxel_count³ 个体素，检查 12 条边是否穿过 isosurface，
// 若是则二分查找精确交叉点并计算法向。

struct VoxelEdgeCrossPoint {
    position: vec3<f32>,
    _pad0: u32,
    normal: vec3<f32>,
    _pad1: u32,
}

struct TerrainChunkInfo {
    chunk_min: vec3<f32>,
    voxel_size: f32,
    voxel_count: u32,
    terrain_size: f32,
    seed: u32,
    _pad: vec2<u32>,
}

@group(0) @binding(0) var<uniform> chunk_info: TerrainChunkInfo;
@group(0) @binding(1) var<storage, read> density: array<f32>;
@group(0) @binding(2) var<storage, read_write> cross_points: array<u32>;

// ── density query ──

fn grid_idx(x: u32, y: u32, z: u32) -> u32 {
    let n = chunk_info.voxel_count + 1u;
    return x + y * n + z * n * n;
}

fn read_density(idx: u32) -> f32 {
    return density[idx];
}

fn grid_pos(idx: u32) -> vec3<f32> {
    let n = chunk_info.voxel_count + 1u;
    let z = idx / (n * n);
    let r = idx % (n * n);
    let y = r / n;
    let x = r % n;
    return chunk_info.chunk_min + vec3<f32>(f32(x), f32(y), f32(z)) * chunk_info.voxel_size;
}

// ── edge definitions: 方向轴, 起点偏移 (相对于 voxel min corner) ──
// 12 条边，按 X→Y→Z 顺序。每条边 = (axis, offset_of_corner_0)

const EDGE_DIRS: array<u32, 12> = array<u32, 12>(0u, 0u, 0u, 0u,  1u, 1u, 1u, 1u,  2u, 2u, 2u, 2u);
const EDGE_CORNERS: array<vec3<u32>, 12> = array<vec3<u32>, 12>(
    vec3(0u, 0u, 0u), vec3(0u, 0u, 1u), vec3(0u, 1u, 0u), vec3(0u, 1u, 1u), // X 轴边
    vec3(0u, 0u, 0u), vec3(1u, 0u, 0u), vec3(0u, 0u, 1u), vec3(1u, 0u, 1u), // Y 轴边
    vec3(0u, 0u, 0u), vec3(1u, 0u, 0u), vec3(0u, 1u, 0u), vec3(1u, 1u, 0u), // Z 轴边
);

// 存储交叉点到 buffer（pack as u32 — position + normal 用 f32 原始字节）
fn write_cross_point(edge_id: u32, pos: vec3<f32>, normal: vec3<f32>) {
    let base = edge_id * 8u; // 8 × u32 = 32 bytes per cross point
    cross_points[base + 0u] = bitcast<u32>(pos.x);
    cross_points[base + 1u] = bitcast<u32>(pos.y);
    cross_points[base + 2u] = bitcast<u32>(pos.z);
    cross_points[base + 3u] = 0u; // pad
    cross_points[base + 4u] = bitcast<u32>(normal.x);
    cross_points[base + 5u] = bitcast<u32>(normal.y);
    cross_points[base + 6u] = bitcast<u32>(normal.z);
    cross_points[base + 7u] = 0u; // pad
}

// 中心差分法向估算
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

// 世界坐标 → 最近网格点 index（仅用于法向估计）
fn grid_idx_from_world(p: vec3<f32>) -> u32 {
    let rel = (p - chunk_info.chunk_min) / chunk_info.voxel_size;
    let n = chunk_info.voxel_count;
    let x = clamp(u32(round(rel.x)), 0u, n);
    let y = clamp(u32(round(rel.y)), 0u, n);
    let z = clamp(u32(round(rel.z)), 0u, n);
    return grid_idx(x, y, z);
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let vc = chunk_info.voxel_count;
    if gid.x >= vc || gid.y >= vc || gid.z >= vc { return; }

    let base = grid_idx(gid.x, gid.y, gid.z);

    for (var e = 0u; e < 12u; e++) {
        let axis = EDGE_DIRS[e];
        let corner = EDGE_CORNERS[e];

        // 边两端点的网格坐标 + density
        let c0_x = gid.x + corner.x;
        let c0_y = gid.y + corner.y;
        let c0_z = gid.z + corner.z;
        let mut c1_x = c0_x;
        let mut c1_y = c0_y;
        let mut c1_z = c0_z;
        if axis == 0u { c1_x += 1u; }
        else if axis == 1u { c1_y += 1u; }
        else { c1_z += 1u; }

        if c1_x > vc || c1_y > vc || c1_z > vc { continue; }

        let d0 = read_density(grid_idx(c0_x, c0_y, c0_z));
        let d1 = read_density(grid_idx(c1_x, c1_y, c1_z));

        // 符号相同 → 无交叉
        if (d0 > 0.0) == (d1 > 0.0) { continue; }

        // 二分查找交叉点
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

        let edge_id = (gid.x + gid.y * vc + gid.z * vc * vc) * 12u + e;
        write_cross_point(edge_id, cross_pos, normal);
    }
}
