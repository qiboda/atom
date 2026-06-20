// Pass 3: 全局 QEF Solve — 对每个分配了 vertex 的 voxel，
// 收集其 crossing edge 的 cross point + normal，求解 QEF 最小二乘，
// 写入 compacted vertex buffer。

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
@group(0) @binding(5) var<storage, read_write> counters: array<u32>;

// ── cross point 读取 ──

fn read_cross_pos(edge_id: u32) -> vec3<f32> {
    let b = edge_id * 8u;
    return vec3(bitcast<f32>(cross[b]),
                bitcast<f32>(cross[b + 1u]),
                bitcast<f32>(cross[b + 2u]));
}

fn read_cross_normal(edge_id: u32) -> vec3<f32> {
    let b = edge_id * 8u + 4u;
    return vec3(bitcast<f32>(cross[b]),
                bitcast<f32>(cross[b + 1u]),
                bitcast<f32>(cross[b + 2u]));
}

fn has_cross(edge_id: u32) -> bool {
    let b = edge_id * 8u;
    return abs(bitcast<f32>(cross[b]))
         + abs(bitcast<f32>(cross[b + 1u]))
         + abs(bitcast<f32>(cross[b + 2u])) > 0.001;
}

// ── 3×3 Cramer's rule + 奇异性检测 ──

fn det3(m00: f32, m01: f32, m02: f32,
        m10: f32, m11: f32, m12: f32,
        m20: f32, m21: f32, m22: f32) -> f32 {
    return m00*(m11*m22 - m12*m21) - m01*(m10*m22 - m12*m20) + m02*(m10*m21 - m11*m20);
}

fn solve3(a00: f32, a01: f32, a02: f32,
          a11: f32, a12: f32, a22: f32,
          b0: f32, b1: f32, b2: f32) -> vec3<f32> {
    let d = det3(a00,a01,a02, a01,a11,a12, a02,a12,a22);
    if abs(d) < 1e-5f { return vec3(0f); } // 奇异矩阵 → 零向量（后续 fallback 到质心）
    return vec3(
        det3(b0,a01,a02, b1,a11,a12, b2,a12,a22) / d,
        det3(a00,b0,a02, a01,b1,a12, a02,b2,a22) / d,
        det3(a00,a01,b0, a01,a11,b1, a02,a12,b2) / d,
    );
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let gs = info.grid_size;
    if gid.x >= gs || gid.y >= gs || gid.z >= gs { return; }

    let voxel_idx = gid.x + gid.y * gs + gid.z * gs * gs;
    let vi = voxel_alloc[voxel_idx];
    if vi == ~0u { return; } // 此 voxel 无顶点

    // 收集 crossing edge 的约束（位置 + 法线）
    var a00=0f; var a01=0f; var a02=0f;
    var a11=0f; var a12=0f; var a22=0f;
    var b0=0f; var b1=0f; var b2=0f;
    var ncross = 0u;
    var avg_pos = vec3(0f);
    var avg_norm = vec3(0f);

    for (var e = 0u; e < 12u; e++) {
        let eid = voxel_idx * 12u + e;
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

    // ncross > 0 由 vertex_alloc pass 保证
    var vp = solve3(a00,a01,a02, a11,a12,a22, b0,b1,b2);
    let centroid = avg_pos / f32(ncross);

    // fallback: 奇异矩阵 → 质心
    if length(vp) < 0.0001f { vp = centroid; }
    // 安全钳: QEF 解离群 → 质心
    else if length(vp - centroid) > info.voxel_size * 2.0 { vp = centroid; }

    let vn = normalize(avg_norm / f32(ncross));
    vertices[vi] = TerrainChunkVertex(vp, 0u, vn, 0u);
}
