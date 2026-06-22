// Step 0: SDF Fill — 以观察者为中心填充全局密度 grid
// density = y - height_at(x,z)

struct GlobalUniforms {
    grid_min: vec3<f32>,
    pad0: u32,
    voxel_size: f32,
    grid_size: u32,    // grid points per axis = n-1
    pad1: vec2<u32>,
}

@group(0) @binding(0) var<uniform> info: GlobalUniforms;
@group(0) @binding(1) var<storage, read_write> density: array<f32>;

fn height_at(xz: vec2<f32>) -> f32 {
    return sin(xz.x * 0.08) * cos(xz.y * 0.08) * 8.0 - 24.0;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let n = info.grid_size + 1u;
    if gid.x >= n || gid.y >= n || gid.z >= n { return; }
    let idx = gid.x + gid.y * n + gid.z * n * n;
    let pos = info.grid_min + vec3<f32>(f32(gid.x), f32(gid.y), f32(gid.z)) * info.voxel_size;
    density[idx] = pos.y - height_at(pos.xz);
}
