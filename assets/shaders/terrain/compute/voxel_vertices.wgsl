// Pass 1: 密度场计算
// 对 (voxel_count+1)³ 个网格点计算 density = y - noise_height(x,z)

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
@group(0) @binding(1) var<storage, read_write> density: array<f32>;

// ── simple hash noise (替代 OpenSimplex，MVP) ──

fn hash22(p: vec2<f32>) -> f32 {
    let n = fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
    return n * 2.0 - 1.0;
}

fn value_noise_2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // smoothstep
    return mix(
        mix(hash22(i + vec2(0.0, 0.0)), hash22(i + vec2(1.0, 0.0)), u.x),
        mix(hash22(i + vec2(0.0, 1.0)), hash22(i + vec2(1.0, 1.0)), u.x),
        u.y,
    );
}

fn fbm_2d(p: vec2<f32>, octaves: u32) -> f32 {
    var val = 0.0;
    var amp = 1.0;
    var freq = 1.0;
    var max_val = 0.0;
    var pp = p;
    for (var i = 0u; i < octaves; i++) {
        val += value_noise_2d(pp * freq) * amp;
        max_val += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    return val / max_val;
}

fn height_at(xz: vec2<f32>) -> f32 {
    let h1 = fbm_2d(xz * 0.02, 3u) * 20.0;
    let h2 = fbm_2d(xz * 0.08, 3u) * 5.0;
    let h3 = fbm_2d(xz * 0.25, 3u) * 1.0;
    return h1 + h2 + h3;
}

fn world_pos(voxel_idx: u32) -> vec3<f32> {
    let n = chunk_info.voxel_count;
    let z = voxel_idx / ((n + 1u) * (n + 1u));
    let r = voxel_idx % ((n + 1u) * (n + 1u));
    let y = r / (n + 1u);
    let x = r % (n + 1u);
    return chunk_info.chunk_min + vec3<f32>(f32(x), f32(y), f32(z)) * chunk_info.voxel_size;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let n = chunk_info.voxel_count + 1u;
    if gid.x >= n || gid.y >= n || gid.z >= n { return; }
    let idx = gid.x + gid.y * n + gid.z * n * n;
    let pos = world_pos(idx);
    density[idx] = pos.y - height_at(pos.xz);
}
