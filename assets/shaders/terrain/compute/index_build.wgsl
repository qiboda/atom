// Pass 4: 全局 Index Build — 对每个 voxel 的三条"主"边（owned edges），
// 生成 quad 并写入 compacted index buffer。
//
// 每条 edge 由 4 个体素共享，quad 只由"主"voxel 生成一次：
//   edge 的 corner == (0,0,0) → 该 voxel 是主 voxel。
//   即 edges 0(X), 4(Y), 8(Z) — 每 voxel 检查 3 条边而非 12 条。
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

fn read_vertex_pos(vi: u32) -> vec3<f32> {
    return vertices[vi].position;
}

// ── 三条 owned edges ──
// edge 0: X-axis, corner=(0,0,0)
// edge 4: Y-axis, corner=(0,0,0)
// edge 8: Z-axis, corner=(0,0,0)

const OWNED_EDGES: array<u32, 3> = array<u32, 3>(0u, 4u, 8u);
const OWNED_AXIS: array<u32, 3> = array<u32, 3>(0u, 1u, 2u);

// ── quad voxels ──
// 返回共享该 edge 的 4 个体素的局部坐标（相对当前 voxel）

fn quad_voxels(axis: u32, dx: i32, dy: i32, dz: i32) -> array<vec4<i32>, 4> {
    // 返回 4 个 (vx, vy, vz, 1) 的整数坐标
    if axis == 0u {
        // X-edge: 4 voxels share the X-direction edge
        return array(
            vec4(dx, dy-1, dz-1, 1),
            vec4(dx, dy,   dz-1, 1),
            vec4(dx, dy,   dz,   1),
            vec4(dx, dy-1, dz,   1),
        );
    } else if axis == 1u {
        // Y-edge
        return array(
            vec4(dx-1, dy, dz-1, 1),
            vec4(dx,   dy, dz-1, 1),
            vec4(dx,   dy, dz,   1),
            vec4(dx-1, dy, dz,   1),
        );
    } else {
        // Z-edge
        return array(
            vec4(dx-1, dy-1, dz, 1),
            vec4(dx,   dy-1, dz, 1),
            vec4(dx,   dy,   dz, 1),
            vec4(dx-1, dy,   dz, 1),
        );
    }
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let gs = info.grid_size;
    if gid.x >= gs || gid.y >= gs || gid.z >= gs { return; }

    let voxel_idx = gid.x + gid.y * gs + gid.z * gs * gs;
    let vx = gid.x; let vy = gid.y; let vz = gid.z;

    // 只检查 3 条 owned edges（避免 quad 重复）
    for (var ei = 0u; ei < 3u; ei++) {
        let e = OWNED_EDGES[ei];
        let axis = OWNED_AXIS[ei];

        let edge_id = voxel_idx * 12u + e;
        if !has_cross(edge_id) { continue; }

        // 计算 4 个 quad voxels
        let qv = quad_voxels(axis, i32(vx), i32(vy), i32(vz));

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

        // winding: 默认 face_n = (+X, -Y, +Z)，heightfield 全部翻转
        // 翻转顺序: vi0-vi2-vi1, vi0-vi3-vi2
        indices[base + 0u] = vidx[0];
        indices[base + 1u] = vidx[2];
        indices[base + 2u] = vidx[1];
        indices[base + 3u] = vidx[0];
        indices[base + 4u] = vidx[3];
        indices[base + 5u] = vidx[2];
    }
}
