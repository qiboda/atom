// Pass 4: 全局 Index Build — 对每个 voxel 的全部 12 条边，
// 判断是否为该 edge 的 canonical owner（或 canonical owner 在 grid 之外），
// 是则生成 quad 并写入 compacted index buffer。
//
// 去重规则：
//   edge 的 canonical owner = (vx + corner.x, vy + corner.y, vz + corner.z)
//   - 若 canonical owner 在负向 overlap zone (坐标 0 或 1) 且邻居存在 → 跳过（邻居负责）
//   - 若 canonical owner 在 grid 内且 ≠ 当前 voxel → 跳过（由 owner 生成）
//   - 否则 → 当前 voxel 生成 quad
//   这保证每条 edge 恰好由一个 voxel 生成 quad，包括 grid 边界和 chunk 边界上的 edge。
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
    neighbor_mask: u32,
    pad3: u32,
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

// 读取 edge crossing 的 flip flag（d1 < 0）
fn read_flip(edge_id: u32) -> bool {
    let b = edge_id * 8u + 3u;
    return bitcast<f32>(cross[b]) > 0.0;
}

fn get_vertex_index(vx: u32, vy: u32, vz: u32) -> u32 {
    let gs = info.grid_size;
    if vx >= gs || vy >= gs || vz >= gs { return ~0u; }
    return voxel_alloc[vx + vy * gs + vz * gs * gs];
}

// ── quad voxels ──
// 返回共享该 edge 的 4 个体素坐标（以 edge 起始 corner 为基准）

fn quad_voxels(axis: u32, cx: u32, cy: u32, cz: u32) -> array<vec3<i32>, 4> {
    let x = i32(cx); let y = i32(cy); let z = i32(cz);
    if axis == 0u {
        return array(vec3(x, y-1, z-1), vec3(x, y, z-1), vec3(x, y, z), vec3(x, y-1, z));
    } else if axis == 1u {
        return array(vec3(x-1, y, z-1), vec3(x, y, z-1), vec3(x, y, z), vec3(x-1, y, z));
    } else {
        return array(vec3(x-1, y-1, z), vec3(x, y-1, z), vec3(x, y, z), vec3(x-1, y, z));
    }
}

// ── 跨 chunk canonical-owner 检查 ──

fn has_neighbor(axis_bit: u32) -> bool {
    return (info.neighbor_mask & (1u << axis_bit)) != 0u;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let gs = info.grid_size;
    if gid.x >= gs || gid.y >= gs || gid.z >= gs { return; }

    let voxel_idx = gid.x + gid.y * gs + gid.z * gs * gs;
    let vx = gid.x; let vy = gid.y; let vz = gid.z;

    for (var e = 0u; e < 12u; e++) {
        let corner = EDGE_CORNERS[e];
        let axis = EDGE_DIRS[e];

        var cx = vx + corner.x;
        var cy = vy + corner.y;
        var cz = vz + corner.z;

        if cx <= 1u && has_neighbor(1u) { continue; }
        if cy <= 1u && has_neighbor(3u) { continue; }
        if cz <= 1u && has_neighbor(5u) { continue; }
        // chunk 内去重：canonical owner 在 grid 内且 ≠ 当前 voxel → 跳过
        let owner_exists = cx < gs && cy < gs && cz < gs;
        if owner_exists && (cx != vx || cy != vy || cz != vz) {
            continue;
        }

        let edge_id = voxel_idx * 12u + e;
        if !has_cross(edge_id) { continue; }

        let qv = quad_voxels(axis, cx, cy, cz);

        var fixed_slots: array<u32, 4>;
        var valid_mask: u32 = 0u;
        for (var i = 0u; i < 4u; i++) {
            let qi = u32(qv[i].x);
            let qj = u32(qv[i].y);
            let qk = u32(qv[i].z);

            if qi >= gs || qj >= gs || qk >= gs { continue; }

            let slot = qi + qj * gs + qk * gs * gs;
            let ci = voxel_alloc[slot];
            if ci != ~0u {
                fixed_slots[i] = ci;
                valid_mask |= 1u << i;
            }
        }

        // Generate triangles from valid vertices
        let flip = read_flip(edge_id);
        let use_flip = select(!flip, flip, axis == 1u);
        if use_flip {
            if valid_mask == 15u {
                let base = atomicAdd(&counters[1], 6u);
                indices[base + 0u] = fixed_slots[0];
                indices[base + 1u] = fixed_slots[2];
                indices[base + 2u] = fixed_slots[1];
                indices[base + 3u] = fixed_slots[0];
                indices[base + 4u] = fixed_slots[3];
                indices[base + 5u] = fixed_slots[2];
            } else {
                var tri: array<u32, 3>;
                var t = 0u;
                for (var i = 0u; i < 4u && t < 3u; i++) {
                    if (valid_mask & (1u << i)) != 0u {
                        tri[t] = fixed_slots[i];
                        t += 1u;
                    }
                }
                if t >= 3u {
                    let base = atomicAdd(&counters[1], 3u);
                    indices[base + 0u] = tri[0];
                    indices[base + 1u] = tri[2];
                    indices[base + 2u] = tri[1];
                }
            }
        } else {
            if valid_mask == 15u {
                let base = atomicAdd(&counters[1], 6u);
                indices[base + 0u] = fixed_slots[0];
                indices[base + 1u] = fixed_slots[1];
                indices[base + 2u] = fixed_slots[2];
                indices[base + 3u] = fixed_slots[0];
                indices[base + 4u] = fixed_slots[2];
                indices[base + 5u] = fixed_slots[3];
            } else {
                var tri: array<u32, 3>;
                var t = 0u;
                for (var i = 0u; i < 4u && t < 3u; i++) {
                    if (valid_mask & (1u << i)) != 0u {
                        tri[t] = fixed_slots[i];
                        t += 1u;
                    }
                }
                if t >= 3u {
                    let base = atomicAdd(&counters[1], 3u);
                    indices[base + 0u] = tri[0];
                    indices[base + 1u] = tri[1];
                    indices[base + 2u] = tri[2];
                }
            }
        }
    }
}
