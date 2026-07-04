// Pass 1: 全局 Edge Detection — 观察者中心，无 chunk 边界。
// 对 grid_size³ 个体素检测 12 条边的 sign change，
// edge_id = voxel_idx * 12 + local_edge（全局唯一）。
//
// 输出到 cross buffer，每个 edge 占 8×u32 (pos + flip + normal + pad)。

struct GlobalUniforms {
    grid_min: vec3<f32>,
    pad0: u32,
    voxel_size: f32,
    grid_size: u32,       // voxels per axis (density grid is grid_size+2)
    pad1: vec2<u32>,
    neighbor_mask: u32,
    pad3: u32,
}

@group(0) @binding(0) var<uniform> info: GlobalUniforms;
@group(0) @binding(1) var<storage, read_write> density: array<f32>;
@group(0) @binding(2) var<storage, read_write> cross: array<u32>;

// ── density grid 索引 ──
// density grid 为 (grid_size+2)³ 个采样点

fn grid_idx(gx: u32, gy: u32, gz: u32) -> u32 {
    let n = info.grid_size + 2u;
    return gx + gy * n + gz * n * n;
}

fn read_density(idx: u32) -> f32 { return density[idx]; }

fn world_pos(gx: u32, gy: u32, gz: u32) -> vec3<f32> {
    return info.grid_min
        + vec3<f32>(f32(gx), f32(gy), f32(gz)) * info.voxel_size;
}

// ── 三线性插值：在连续密度场上采样（替代 grid_idx_from_world + round） ──

fn trilinear_sample(p: vec3<f32>) -> f32 {
    let rel = (p - info.grid_min) / info.voxel_size;
    let n = info.grid_size + 2u;

    let fx = clamp(rel.x, 0.0, f32(n - 1u));
    let fy = clamp(rel.y, 0.0, f32(n - 1u));
    let fz = clamp(rel.z, 0.0, f32(n - 1u));

    let ix = u32(floor(fx));
    let iy = u32(floor(fy));
    let iz = u32(floor(fz));

    let jx = min(ix + 1u, n - 1u);
    let jy = min(iy + 1u, n - 1u);
    let jz = min(iz + 1u, n - 1u);

    let wx = fx - f32(ix);
    let wy = fy - f32(iy);
    let wz = fz - f32(iz);

    // 8 个角点
    let d000 = read_density(grid_idx(ix, iy, iz));
    let d100 = read_density(grid_idx(jx, iy, iz));
    let d010 = read_density(grid_idx(ix, jy, iz));
    let d110 = read_density(grid_idx(jx, jy, iz));
    let d001 = read_density(grid_idx(ix, iy, jz));
    let d101 = read_density(grid_idx(jx, iy, jz));
    let d011 = read_density(grid_idx(ix, jy, jz));
    let d111 = read_density(grid_idx(jx, jy, jz));

    // 三线性插值
    let d00 = d000 + (d100 - d000) * wx;
    let d01 = d001 + (d101 - d001) * wx;
    let d10 = d010 + (d110 - d010) * wx;
    let d11 = d011 + (d111 - d011) * wx;

    let d0 = d00 + (d10 - d00) * wy;
    let d1 = d01 + (d11 - d01) * wy;

    return d0 + (d1 - d0) * wz;
}

// ── 法线估计（中心差分，使用三线性插值）──

fn estimate_normal(p: vec3<f32>) -> vec3<f32> {
    let h = info.voxel_size * 0.5;
    let dx = trilinear_sample(p + vec3(h, 0.0, 0.0))
           - trilinear_sample(p - vec3(h, 0.0, 0.0));
    let dy = trilinear_sample(p + vec3(0.0, h, 0.0))
           - trilinear_sample(p - vec3(0.0, h, 0.0));
    let dz = trilinear_sample(p + vec3(0.0, 0.0, h))
           - trilinear_sample(p - vec3(0.0, 0.0, h));
    return normalize(vec3(dx, dy, dz));
}

// ── 12 条边定义 ──

const EDGE_DIRS: array<u32, 12> = array<u32, 12>(
    0u,0u,0u,0u, 1u,1u,1u,1u, 2u,2u,2u,2u,
);

const EDGE_CORNERS: array<vec3<u32>, 12> = array<vec3<u32>, 12>(
    vec3(0u,0u,0u), vec3(0u,0u,1u), vec3(0u,1u,0u), vec3(0u,1u,1u),
    vec3(0u,0u,0u), vec3(1u,0u,0u), vec3(0u,0u,1u), vec3(1u,0u,1u),
    vec3(0u,0u,0u), vec3(1u,0u,0u), vec3(0u,1u,0u), vec3(1u,1u,0u),
);

// ── cross point 写入 ──
// 每个 edge: 8×u32 [pos.x, pos.y, pos.z, flip, normal.x, normal.y, normal.z, pad]

fn write_cross(edge_id: u32, pos: vec3<f32>, normal: vec3<f32>, flip: bool) {
    let base = edge_id * 8u;
    cross[base + 0u] = bitcast<u32>(pos.x);
    cross[base + 1u] = bitcast<u32>(pos.y);
    cross[base + 2u] = bitcast<u32>(pos.z);
    cross[base + 3u] = bitcast<u32>(f32(flip));
    cross[base + 4u] = bitcast<u32>(normal.x);
    cross[base + 5u] = bitcast<u32>(normal.y);
    cross[base + 6u] = bitcast<u32>(normal.z);
    cross[base + 7u] = 0u;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let gs = info.grid_size;
    if gid.x >= gs || gid.y >= gs || gid.z >= gs { return; }

    let voxel_idx = gid.x + gid.y * gs + gid.z * gs * gs;

    for (var e = 0u; e < 12u; e++) {
        let axis = EDGE_DIRS[e];
        let corner = EDGE_CORNERS[e];

        // edge 两端 corner 在 density grid 中的坐标
        let c0x = gid.x + corner.x;
        let c0y = gid.y + corner.y;
        let c0z = gid.z + corner.z;
        var c1x = c0x; var c1y = c0y; var c1z = c0z;
        if axis == 0u { c1x += 1u; }
        else if axis == 1u { c1y += 1u; }
        else { c1z += 1u; }

        let n = gs + 2u; // density grid points per axis
        let edge_id = voxel_idx * 12u + e;

        // 边界外或无 sign change → 清零 cross entry，防止 stale 数据被误判
        if c1x >= n || c1y >= n || c1z >= n {
            write_cross(edge_id, vec3(0.0), vec3(0.0), false);
            continue;
        }

        let d0 = read_density(grid_idx(c0x, c0y, c0z));
        let d1 = read_density(grid_idx(c1x, c1y, c1z));
        if d0 * d1 > 0.0 {
            write_cross(edge_id, vec3(0.0), vec3(0.0), false);
            continue;
        }

        // binary search 定位交叉点（8 次迭代）
        let p0 = world_pos(c0x, c0y, c0z);
        let p1 = world_pos(c1x, c1y, c1z);
        var lo = p0; var hi = p1; var dlo = d0;

        for (var iter = 0u; iter < 8u; iter++) {
            let mid = (lo + hi) * 0.5;
            let dmid = trilinear_sample(mid);
            if (dmid > 0.0) == (dlo > 0.0) { lo = mid; dlo = dmid; }
            else { hi = mid; }
        }
        let cross_pos = (lo + hi) * 0.5;
        let normal = estimate_normal(cross_pos);

        write_cross(edge_id, cross_pos, normal, d1 < 0.0);
    }
}
