// Pass 4: 三角索引生成（DC quad，固定 slot 无 atomics）
// 每个 voxel 最多 6 个索引写入 indices[voxel_idx * 6 + ...]

struct TerrainChunkVertex {
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
@group(0) @binding(2) var<storage, read> cross_points: array<u32>;
@group(0) @binding(3) var<storage, read> vertices: array<TerrainChunkVertex>;
@group(0) @binding(4) var<storage, read_write> indices: array<u32>;

fn has_cross(edge_id: u32) -> bool {
    let b = edge_id * 8u;
    return abs(bitcast<f32>(cross_points[b]))
         + abs(bitcast<f32>(cross_points[b + 1u]))
         + abs(bitcast<f32>(cross_points[b + 2u])) > 0.001;
}

fn has_vertex(vx: u32, vy: u32, vz: u32) -> bool {
    let vc = chunk_info.voxel_count;
    if vx >= vc || vy >= vc || vz >= vc { return false; }
    return length(vertices[vx + vy * vc + vz * vc * vc].position) > 0.0001;
}

fn voxel_edge_id(vx: u32, vy: u32, vz: u32, e: u32) -> u32 {
    let vc = chunk_info.voxel_count;
    return (vx + vy * vc + vz * vc * vc) * 12u + e;
}

const EDGE_AXIS: array<u32, 12> = array(0u,0u,0u,0u, 1u,1u,1u,1u, 2u,2u,2u,2u);
const EDGE_CORNER: array<vec3<u32>, 12> = array(
    vec3(0u,0u,0u), vec3(0u,0u,1u), vec3(0u,1u,0u), vec3(0u,1u,1u),
    vec3(0u,0u,0u), vec3(1u,0u,0u), vec3(0u,0u,1u), vec3(1u,0u,1u),
    vec3(0u,0u,0u), vec3(1u,0u,0u), vec3(0u,1u,0u), vec3(1u,1u,0u),
);

// 4 个相邻 voxel（axis == edge 方向, cx,cy,cz == edge 起点网格坐标）
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

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let vc = chunk_info.voxel_count;
    if gid.x >= vc || gid.y >= vc || gid.z >= vc { return; }

    for (var e = 0u; e < 12u; e++) {
        let axis = EDGE_AXIS[e];
        let corner = EDGE_CORNER[e];
        let cx = gid.x + corner.x;
        let cy = gid.y + corner.y;
        let cz = gid.z + corner.z;
        let eid = voxel_edge_id(gid.x, gid.y, gid.z, e);
        if !has_cross(eid) { continue; }

        let qv = quad_voxels(axis, cx, cy, cz);
        var all = true;
        for (var i = 0u; i < 4u; i++) {
            let q = qv[i];
            if q.x < 0i || q.y < 0i || q.z < 0i
                || q.x >= i32(vc) || q.y >= i32(vc) || q.z >= i32(vc)
                || !has_vertex(u32(q.x), u32(q.y), u32(q.z)) {
                all = false; break;
            }
        }
        if !all { continue; }

        let vi0 = u32(qv[0].x) + u32(qv[0].y) * vc + u32(qv[0].z) * vc * vc;
        let vi1 = u32(qv[1].x) + u32(qv[1].y) * vc + u32(qv[1].z) * vc * vc;
        let vi2 = u32(qv[2].x) + u32(qv[2].y) * vc + u32(qv[2].z) * vc * vc;
        let vi3 = u32(qv[3].x) + u32(qv[3].y) * vc + u32(qv[3].z) * vc * vc;

        // 写入固定 slot: gid 对应的 voxel，最多 6 个索引
        let base = (gid.x + gid.y * vc + gid.z * vc * vc) * 6u;
        indices[base + 0u] = vi0;
        indices[base + 1u] = vi1;
        indices[base + 2u] = vi2;
        indices[base + 3u] = vi0;
        indices[base + 4u] = vi2;
        indices[base + 5u] = vi3;
    }
}
