// Pass 2: Vertex Allocation — atomic count + compact.
// 统计每个 voxel 的 crossing edge 数量，为有交叉的 voxel 分配 vertex_index。
//
// voxel_alloc[voxel_idx] = vertex_index (compact) 或 ~0u (无交叉)
// counters[0] = total_vertices（原子递增）

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
@group(0) @binding(5) var<storage, read_write> counters: array<atomic<u32>>;

fn has_cross(edge_id: u32) -> bool {
    let b = edge_id * 8u;
    return abs(bitcast<f32>(cross[b]))
         + abs(bitcast<f32>(cross[b + 1u]))
         + abs(bitcast<f32>(cross[b + 2u])) > 0.001;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let gs = info.grid_size;
    if gid.x >= gs || gid.y >= gs || gid.z >= gs { return; }

    let voxel_idx = gid.x + gid.y * gs + gid.z * gs * gs;

    // 统计该 voxel 有多少条边穿过 isosurface
    var ncross = 0u;
    for (var e = 0u; e < 12u; e++) {
        if has_cross(voxel_idx * 12u + e) { ncross += 1u; }
    }

    if ncross == 0u {
        voxel_alloc[voxel_idx] = ~0u; // sentinel: 无顶点
        return;
    }

    // atomic 分配 vertex index
    let vi = atomicAdd(&counters[0], 1u);
    voxel_alloc[voxel_idx] = vi;
}
