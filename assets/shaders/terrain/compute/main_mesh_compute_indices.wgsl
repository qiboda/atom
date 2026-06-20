// Pass 4: 三角索引生成（DC quad，固定 slot 无 atomics）
// 仅内层 [1, vc]³ voxel 生成 quad，可引用双边 shell 顶点

struct TerrainChunkVertex {
    position: vec3<f32>,
    pad0: u32,
    normal: vec3<f32>,
    pad1: u32,
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
@group(0) @binding(2) var<storage, read_write> cross_points: array<u32>;
@group(0) @binding(3) var<storage, read_write> vertices: array<TerrainChunkVertex>;
@group(0) @binding(4) var<storage, read_write> indices: array<u32>;

fn has_cross(edge_id: u32) -> bool {
    let b = edge_id * 8u;
    return abs(bitcast<f32>(cross_points[b]))
         + abs(bitcast<f32>(cross_points[b + 1u]))
         + abs(bitcast<f32>(cross_points[b + 2u])) > 0.001;
}

fn read_cross_normal(edge_id: u32) -> vec3<f32> {
    let b = edge_id * 8u + 4u;
    return vec3(bitcast<f32>(cross_points[b]),
                bitcast<f32>(cross_points[b + 1u]),
                bitcast<f32>(cross_points[b + 2u]));
}

fn vertex_pos(vi: u32) -> vec3<f32> {
    let vv = chunk_info.voxel_count + 2u;
    if vi >= vv * vv * vv { return vec3(0f); }
    return vertices[vi].position;
}

fn has_vertex(vx: u32, vy: u32, vz: u32) -> bool {
    let vv = chunk_info.voxel_count + 2u; // 双边 shell
    if vx >= vv || vy >= vv || vz >= vv { return false; }
    return length(vertices[vx + vy * vv + vz * vv * vv].position) > 0.0001;
}

fn voxel_edge_id(vx: u32, vy: u32, vz: u32, e: u32) -> u32 {
    let vv = chunk_info.voxel_count + 2u;
    return (vx + vy * vv + vz * vv * vv) * 12u + e;
}

const EDGE_AXIS: array<u32, 12> = array(0u,0u,0u,0u, 1u,1u,1u,1u, 2u,2u,2u,2u);
const EDGE_CORNER: array<vec3<u32>, 12> = array(
    vec3(0u,0u,0u), vec3(0u,0u,1u), vec3(0u,1u,0u), vec3(0u,1u,1u),
    vec3(0u,0u,0u), vec3(1u,0u,0u), vec3(0u,0u,1u), vec3(1u,0u,1u),
    vec3(0u,0u,0u), vec3(1u,0u,0u), vec3(0u,1u,0u), vec3(1u,1u,0u),
);

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
    let vv = vc + 2u;

    // 仅内层 voxel [1, vc] 生成 quad（shell 只提供顶点引用）
    if gid.x < 1u || gid.x > vc || gid.y < 1u || gid.y > vc || gid.z < 1u || gid.z > vc { return; }

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
                || q.x >= i32(vv) || q.y >= i32(vv) || q.z >= i32(vv)
                || !has_vertex(u32(q.x), u32(q.y), u32(q.z)) {
                all = false; break;
            }
            // 负 shell 且顶点是真实交叉(非 fallback) → 邻居会生成此 quad，本侧跳过
            let vi = u32(q.x) + u32(q.y) * vv + u32(q.z) * vv * vv;
            let vn = vertices[vi].normal;
            let is_real = length(vn) > 0.0001;
            if is_real && (q.x == 0i || q.y == 0i || q.z == 0i) {
                all = false; break;
            }
        }
        if !all { continue; }

        let vi0 = u32(qv[0].x) + u32(qv[0].y) * vv + u32(qv[0].z) * vv * vv;
        let vi1 = u32(qv[1].x) + u32(qv[1].y) * vv + u32(qv[1].z) * vv * vv;
        let vi2 = u32(qv[2].x) + u32(qv[2].y) * vv + u32(qv[2].z) * vv * vv;
        let vi3 = u32(qv[3].x) + u32(qv[3].y) * vv + u32(qv[3].z) * vv * vv;

        // 每 voxel 72 index（12 edge × 6），每个 edge 独立 slot，无覆盖
        let jx = gid.x - 1u;
        let jy = gid.y - 1u;
        let jz = gid.z - 1u;
        let off = (jx + jy * vc + jz * vc * vc) * 72u + e * 6u;

        // winding 矫正：face normal 与 cross normal 对齐
        let cn = read_cross_normal(eid);
        let vpos0 = vertex_pos(vi0);
        let vpos1 = vertex_pos(vi1);
        let vpos2 = vertex_pos(vi2);
        let face_n = cross(vpos1 - vpos0, vpos2 - vpos0);
        if dot(face_n, cn) >= 0.0 {
            indices[off + 0u] = vi0;
            indices[off + 1u] = vi1;
            indices[off + 2u] = vi2;
            indices[off + 3u] = vi0;
            indices[off + 4u] = vi2;
            indices[off + 5u] = vi3;
        } else {
            indices[off + 0u] = vi0;
            indices[off + 1u] = vi2;
            indices[off + 2u] = vi1;
            indices[off + 3u] = vi0;
            indices[off + 4u] = vi3;
            indices[off + 5u] = vi2;
        }
    }
}
