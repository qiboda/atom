// Pass 1: 密度场计算
// 对 (vc+3)³ 个网格点计算 density = y - height_at(x,z)
// 网格索引偏移: grid[0] = chunk_min - voxel_size (负 shell)

struct TerrainChunkVertex {
    position: vec3<f32>,
    pad0: u32,
    normal: vec3<f32>,
    pad1: u32,
}

struct TerrainChunkInfo {
    chunk_min: vec3<f32>,
    pad0: u32,        // align chunk_min to 16 bytes
    voxel_size: f32,
    voxel_count: u32,
    terrain_size: f32,
    seed: u32,
    pad1: vec2<u32>,  // fill to 32 bytes
    pad2: vec2<u32>,  // fill to 48 bytes (multiple of 16)
}

@group(0) @binding(0) var<uniform> chunk_info: TerrainChunkInfo;
@group(0) @binding(1) var<storage, read_write> density: array<f32>;

// ── 测试 pattern: 平滑正弦波 ──

fn height_at(xz: vec2<f32>) -> f32 {
    return sin(xz.x * 0.15) * 6.0 - 24.0;
}

fn world_pos(idx: u32) -> vec3<f32> {
    let dn = chunk_info.voxel_count + 3u; // vc+3 grid points (双边 shell)
    let z = idx / (dn * dn);
    let r = idx % (dn * dn);
    let y = r / dn;
    let x = r % dn;
    // grid[0] = chunk_min - voxel_size
    return chunk_info.chunk_min + (vec3<f32>(f32(x), f32(y), f32(z)) - 1.0) * chunk_info.voxel_size;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dn = chunk_info.voxel_count + 3u;
    if gid.x >= dn || gid.y >= dn || gid.z >= dn { return; }
    let idx = gid.x + gid.y * dn + gid.z * dn * dn;
    let pos = world_pos(idx);
    density[idx] = pos.y - height_at(pos.xz);
}
