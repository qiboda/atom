// Pass 4: 全局 Index Build — 对每个 voxel 的全部 12 条边，
// 判断是否为该 edge 的 canonical owner（或 canonical owner 在 grid 之外），
// 是则生成 quad 并写入 compacted index buffer。
//
// 去重规则：
//   edge 的 canonical owner = (vx + corner.x, vy + corner.y, vz + corner.z)
//   - 若 canonical owner 在 grid 内且 ≠ 当前 voxel → 跳过（由 owner 生成）
//   - 否则 → 当前 voxel 生成 quad
//   这保证每条 edge 恰好由一个 voxel 生成 quad，包括 grid 边界上的 edge。
//
// atomicAdd(&counters[1], 6u) 分配 6 个 index slot。

struct TerrainChunkVertex {
    position: vec3<f32>,
    pad0: u32,
    normal: vec3<f32>,
    pad1: u32,
}

struct GlobalUniforms {
    grid_min: vec3<f32>,
    pad0: u32,
    voxel_size: f32,
    grid_size: u32,
    pad1: vec2<u32>,
}

@group(0) @binding(0) var<uniform> info: GlobalUniforms;
@group(0) @binding(2) var<storage, read_write> cross: array<u32>;
@group(0) @binding(3) var<storage, read_write> voxel_alloc: array<u32>;
@group(0) @binding(4) var<storage, read_write> vertices: array<TerrainChunkVertex>;
@group(0) @binding(5) var<storage, read_write> counters: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> indices: array<u32>;

// ── 12 条边定义 ──

const EDGE_DIRS: array<u32, 12> = array(0u,0u,0u,0u, 1u,1u,1u,1u, 2u,2u,2u,2u);
const EDGE_CORNERS: array<vec3<u32>, 12> = array(
    vec3(0u,0u,0u), vec3(0u,0u,1u), vec3(0u,1u,0u), vec3(0u,1u,1u),
    vec3(0u,0u,0u), vec3(1u,0u,0u), vec3(0u,0u,1u), vec3(1u,0u,1u),
    vec3(0u,0u,0u), vec3(1u,0u,0u), vec3(0u,1u,0u), vec3(1u,1u,0u),
);

// ── cross / vertex 查询 ──

fn has_cross(edge_id: u32) -> bool {
    let b = edge_id * 8u;
    return abs(bitcast<f32>(cross[b]))
         + abs(bitcast<f32>(cross[b + 1u]))
         + abs(bitcast<f32>(cross[b + 2u])) > 0.001;
}

fn get_vertex_index(vx: u32, vy: u32, vz: u32) -> u32 {
    let gs = info.grid_size;
    if vx >= gs || vy >= gs || vz >= gs { return ~0u; }
    return voxel_alloc[vx + vy * gs + vz * gs * gs];
}

// ── quad voxels ──
// 返回共享该 edge 的 4 个体素坐标（以 edge 起始 corner 为基准）

fn quad_voxels(axis: u32, cx: i32, cy: i32, cz: i32) -> array<vec4<i32>, 4> {
    if axis == 0u {
        // X-edge: 4 voxels share the X-direction edge
        return array(
            vec4(cx, cy-1, cz-1, 1),
            vec4(cx, cy,   cz-1, 1),
            vec4(cx, cy,   cz,   1),
            vec4(cx, cy-1, cz,   1),
        );
    } else if axis == 1u {
        // Y-edge
        return array(
            vec4(cx-1, cy, cz-1, 1),
            vec4(cx,   cy, cz-1, 1),
            vec4(cx,   cy, cz,   1),
            vec4(cx-1, cy, cz,   1),
        );
    } else {
        // Z-edge
        return array(
            vec4(cx-1, cy-1, cz, 1),
            vec4(cx,   cy-1, cz, 1),
            vec4(cx,   cy,   cz, 1),
            vec4(cx-1, cy,   cz, 1),
        );
    }
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let gs = info.grid_size;
    if gid.x >= gs || gid.y >= gs || gid.z >= gs { return; }

    if gid.x >= gs || gid.y >= gs || gid.z >= gs { return; }

    let voxel_idx = gid.x + gid.y * gs + gid.z * gs * gs;
    let vx = gid.x; let vy = gid.y; let vz = gid.z;

    // 检查全部 12 条边，通过 canonical owner 规则去重
    for (var e = 0u; e < 12u; e++) {
        let corner = EDGE_CORNERS[e];
        let axis = EDGE_DIRS[e];

        // edge 的起始 corner
        let cx = vx + corner.x;
        let cy = vy + corner.y;
        let cz = vz + corner.z;

        // 去重：若 canonical owner 在 grid 内且 ≠ 当前 voxel → 跳过。
        // canonical owner 会自己生成这个 quad。
        // 注：shell voxel (x=0 等) 现在也生成 quad，由边界检查过滤越界的。
        let owner_exists = cx < gs && cy < gs && cz < gs;
        if owner_exists && (cx != vx || cy != vy || cz != vz) {
            continue; // canonical owner 存在 → 由它生成
        }
        // 否则: canonical owner 不存在（在 grid 外）→ 由当前 voxel 生成

        let edge_id = voxel_idx * 12u + e;
        if !has_cross(edge_id) { continue; }

        // 以 edge 起始 corner 为基准计算 4 个 quad voxels
        let qv = quad_voxels(axis, i32(cx), i32(cy), i32(cz));

        // 查询 4 个 voxel 的 vertex index
        var vidx: array<u32, 4>;
        var all = true;
        for (var i = 0u; i < 4u; i++) {
            let qx = u32(qv[i].x);
            let qy = u32(qv[i].y);
            let qz = u32(qv[i].z);

            // 边界检查: quad voxel 超出 grid → 跳过此 quad
            if qx >= gs || qy >= gs || qz >= gs { all = false; break; }

            let vi = voxel_alloc[qx + qy * gs + qz * gs * gs];
            if vi == ~0u { all = false; break; }
            vidx[i] = vi;
        }
        if !all { continue; }

        // atomic 分配 6 个 index slot
        let base = atomicAdd(&counters[1], 6u);

        // winding: 默认 face_n = (+X, -Y, +Z) → 翻转后 = (-X, +Y, -Z)
        // 对 heightfield (Y 朝上) Y 轴必需翻转。全部翻转（同 Phase 2）。
        indices[base + 0u] = vidx[0];
        indices[base + 1u] = vidx[2];
        indices[base + 2u] = vidx[1];
        indices[base + 3u] = vidx[0];
        indices[base + 4u] = vidx[3];
        indices[base + 5u] = vidx[2];
    }
}
