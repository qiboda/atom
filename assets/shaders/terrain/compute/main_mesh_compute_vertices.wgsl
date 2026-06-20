// Pass 3: QEF 顶点计算 (无 atomics，固定 voxel index 写入)
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
@group(0) @binding(3) var<storage, read_write> vertices: array<TerrainChunkVertex>;

fn read_cross_pos(edge_id: u32) -> vec3<f32> {
    let b = edge_id * 8u;
    return vec3(bitcast<f32>(cross_points[b]),
                bitcast<f32>(cross_points[b + 1u]),
                bitcast<f32>(cross_points[b + 2u]));
}

fn read_cross_normal(edge_id: u32) -> vec3<f32> {
    let b = edge_id * 8u + 4u;
    return vec3(bitcast<f32>(cross_points[b]),
                bitcast<f32>(cross_points[b + 1u]),
                bitcast<f32>(cross_points[b + 2u]));
}

fn has_cross(edge_id: u32) -> bool {
    let b = edge_id * 8u;
    return abs(bitcast<f32>(cross_points[b]))
         + abs(bitcast<f32>(cross_points[b + 1u]))
         + abs(bitcast<f32>(cross_points[b + 2u])) > 0.001;
}

fn det3(m00: f32, m01: f32, m02: f32,
        m10: f32, m11: f32, m12: f32,
        m20: f32, m21: f32, m22: f32) -> f32 {
    return m00*(m11*m22 - m12*m21) - m01*(m10*m22 - m12*m20) + m02*(m10*m21 - m11*m20);
}

fn solve3(a00: f32, a01: f32, a02: f32,
          a11: f32, a12: f32, a22: f32,
          b0: f32, b1: f32, b2: f32) -> vec3<f32> {
    let d = det3(a00,a01,a02, a01,a11,a12, a02,a12,a22);
    if abs(d) < 1e-10f { return vec3(0f); }
    return vec3(
        det3(b0,a01,a02, b1,a11,a12, b2,a12,a22) / d,
        det3(a00,b0,a02, a01,b1,a12, a02,b2,a22) / d,
        det3(a00,a01,b0, a01,a11,b1, a02,a12,b2) / d,
    );
}

fn voxel_edge_id(vx: u32, vy: u32, vz: u32, e: u32) -> u32 {
    let vc = chunk_info.voxel_count;
    return (vx + vy * vc + vz * vc * vc) * 12u + e;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let vc = chunk_info.voxel_count;
    if gid.x >= vc || gid.y >= vc || gid.z >= vc { return; }

    var a00=0f; var a01=0f; var a02=0f;
    var a11=0f; var a12=0f; var a22=0f;
    var b0=0f; var b1=0f; var b2=0f;
    var ncross = 0u;
    var avg_pos = vec3(0f);
    var avg_norm = vec3(0f);

    for (var e = 0u; e < 12u; e++) {
        let eid = voxel_edge_id(gid.x, gid.y, gid.z, e);
        if !has_cross(eid) { continue; }
        let p = read_cross_pos(eid);
        let n = read_cross_normal(eid);
        ncross += 1u;
        avg_pos += p;
        avg_norm += n;
        let d = dot(n, p);
        a00 += n.x*n.x; a01 += n.x*n.y; a02 += n.x*n.z;
        a11 += n.y*n.y; a12 += n.y*n.z; a22 += n.z*n.z;
        b0 += n.x*d; b1 += n.y*d; b2 += n.z*d;
    }

    if ncross == 0u { return; }

    var vp = solve3(a00,a01,a02, a11,a12,a22, b0,b1,b2);
    if length(vp) < 0.0001f { vp = avg_pos / f32(ncross); }
    let vn = normalize(avg_norm / f32(ncross));
    let vi = gid.x + gid.y * vc + gid.z * vc * vc;
    vertices[vi] = TerrainChunkVertex(vp, 0u, vn, 0u);
}
