//! Indirect terrain render — multi-island with temperature-based biomes.
//! Vertex format: position(12) + pad(4) + normal(12) + pad(4) = 32 bytes.

// ── Hash ──
fn hash_u32(x: u32) -> u32 {
    var h = x;
    h = h ^ (h >> 16u); h = h * 0x85ebca6bu;
    h = h ^ (h >> 13u); h = h * 0xc2b2ae35u;
    h = h ^ (h >> 16u);
    return h;
}
fn hash_2d(x: i32, z: i32, seed: u32) -> u32 {
    return hash_u32(seed ^ (u32(x) * 374761393u) ^ (u32(z) * 668265263u));
}
fn hash_f32(h: u32) -> f32 { return f32(h & 0x7fffffu) / f32(0x7fffffu); }
fn noise_2d(x: f32, z: f32, seed: u32) -> f32 {
    let ix = i32(floor(x)); let iz = i32(floor(z));
    let fx = x - f32(ix); let fz = z - f32(iz);
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sz = fz * fz * (3.0 - 2.0 * fz);
    let v00 = hash_f32(hash_2d(ix, iz, seed)) * 2.0 - 1.0;
    let v10 = hash_f32(hash_2d(ix + 1, iz, seed)) * 2.0 - 1.0;
    let v01 = hash_f32(hash_2d(ix, iz + 1, seed)) * 2.0 - 1.0;
    let v11 = hash_f32(hash_2d(ix + 1, iz + 1, seed)) * 2.0 - 1.0;
    return mix(mix(v00, v10, sx), mix(v01, v11, sx), sz);
}
fn fbm(x: f32, z: f32, octaves: u32, persistence: f32, seed: u32) -> f32 {
    var value = 0.0; var amplitude = 1.0; var frequency = 1.0; var max_val = 0.0;
    for (var i = 0u; i < octaves; i++) {
        value += noise_2d(x * frequency, z * frequency, seed + i * 97u) * amplitude;
        max_val += amplitude; amplitude *= persistence; frequency *= 2.0;
    }
    return value / max_val;
}

const CELL_SIZE: f32 = 30.0;
const ISLAND_SPACING: f32 = 500.0;
const ISLAND_PROB: f32 = 0.3;
const ISLAND_MIN_R: f32 = 150.0;
const ISLAND_MAX_R: f32 = 500.0;

struct IslandInfo { center: vec2<f32>, radius: f32, exists: bool, }
fn island_at(gx: i32, gz: i32, seed: u32) -> IslandInfo {
    let h = hash_2d(gx, gz, seed ^ 0x15feed0u);
    let exists = hash_f32(h) < ISLAND_PROB;
    let ox = (hash_f32(h) - 0.5) * ISLAND_SPACING * 0.6;
    let oz = (hash_f32(hash_u32(h)) - 0.5) * ISLAND_SPACING * 0.6;
    let r = ISLAND_MIN_R + hash_f32(hash_u32(h ^ 0x55u)) * (ISLAND_MAX_R - ISLAND_MIN_R);
    var info: IslandInfo;
    info.center = vec2<f32>(f32(gx) * ISLAND_SPACING + ox, f32(gz) * ISLAND_SPACING + oz);
    info.radius = r; info.exists = exists;
    return info;
}

fn cell_seed_pos(cx: i32, cz: i32, seed: u32) -> vec2<f32> {
    let h = hash_2d(cx, cz, seed);
    let ox = (hash_f32(h) - 0.5) * CELL_SIZE * 0.7;
    let oz = (hash_f32(hash_u32(h)) - 0.5) * CELL_SIZE * 0.7;
    return vec2<f32>(f32(cx) * CELL_SIZE + ox, f32(cz) * CELL_SIZE + oz);
}

fn island_falloff(pos: vec2<f32>, seed: u32) -> f32 {
    let gx = i32(floor(pos.x / ISLAND_SPACING));
    let gz = i32(floor(pos.y / ISLAND_SPACING));
    var max_f: f32 = 0.0;
    for (var dx = -1; dx <= 1; dx++) {
        for (var dz = -1; dz <= 1; dz++) {
            let island = island_at(gx + dx, gz + dz, seed);
            if !island.exists { continue; }
            let d_raw = length(pos - island.center);
            let coast1 = fbm(pos.x * 0.018, pos.y * 0.018, 4u, 0.55, seed ^ 0x7f7f7f7fu) * island.radius * 0.35;
            let coast2 = fbm(pos.x * 0.06, pos.y * 0.06, 2u, 0.5, seed ^ 0x3f3f3f3fu) * island.radius * 0.15;
            let coast3 = fbm(pos.x * 0.003, pos.y * 0.003, 2u, 0.5, seed ^ 0x5a5a5a5au) * island.radius * 0.15;
            let d = max(d_raw + coast1 + coast2 + coast3, 0.0);
            if d < island.radius * 1.1 {
                let f = 1.0 - smoothstep(island.radius * 0.5, island.radius, d);
                max_f = max(max_f, f);
            }
        }
    }
    return max_f;
}


// ── Map edge ocean border ──
fn edge_falloff(pos: vec2<f32>) -> f32 {
    let half = 2048.0;
    let margin = 200.0;
    let d = max(abs(pos.x), abs(pos.y)) / (half - margin);
    return 1.0 - smoothstep(0.85, 1.0, d);
}

fn elevation_at(pos: vec2<f32>, seed: u32) -> f32 {
    let wx = pos.x * 0.004; let wz = pos.y * 0.004;
    let elev = (1.0 + fbm(wx, wz, 4u, 0.55, seed)) * 0.55;
    let falloff = island_falloff(pos, seed) * edge_falloff(pos);
    return max(elev * falloff, 0.0);
}

fn temperature_at(pos: vec2<f32>, seed: u32) -> f32 {
    let lat = (pos.y + 2048.0) / 4096.0;
    let noise = fbm(pos.x * 0.0004, pos.y * 0.0004, 3u, 0.55, seed ^ 0xdeedeedu);
    return clamp(lat * 0.55 + noise * 0.22 + 0.5, 0.0, 1.0);
}

fn moisture_at(pos: vec2<f32>, seed: u32) -> f32 {
    let wx = pos.x * 0.005 + 100.0; let wz = pos.y * 0.005 + 100.0;
    return fbm(wx, wz, 3u, 0.55, seed ^ 0xaaaaaaaau) * 0.5 + 0.5;
}

// ── Whittaker: (temp, moist, elev) → biome ──
fn biome_whittaker(temp: f32, moist: f32, elev: f32) -> u32 {
    if elev < 0.05 { return 8u; }
    if elev > 0.7  { if temp < 0.3 { return 0u; } else { return 6u; } }
    if temp < 0.25 {
        if moist < 0.33 { return 1u; } else if moist < 0.66 { return 2u; } else { return 0u; }
    }
    if temp < 0.45 {
        if moist < 0.33 { return 4u; } else if moist < 0.66 { return 2u; } else { return 3u; }
    }
    if temp < 0.65 {
        if moist < 0.2  { return 5u; } else if moist < 0.5 { return 4u; } else if moist < 0.8 { return 3u; } else { return 7u; }
    }
    if moist < 0.15 { return 5u; } else if moist < 0.4 { return 4u; } else { return 7u; }
}
fn surface_type_at(world_xz: vec2<f32>) -> u32 {
    let e = elevation_at(world_xz, 42u);
    let t = temperature_at(world_xz, 42u);
    let m = moisture_at(world_xz, 42u);
    return biome_whittaker(t, m, e);
}

@group(0) @binding(0) var<uniform> clip_from_world: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
};

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = clip_from_world * vec4<f32>(in.position, 1.0);
    out.world_normal = in.normal;
    out.world_pos = in.position;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
#ifdef WIREFRAME
    return vec4<f32>(0.3, 0.9, 0.4, 1.0);
#else
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let n = normalize(in.world_normal);
    let diffuse = max(dot(n, light_dir), 0.0) * 0.7 + 0.3;

    let st = surface_type_at(in.world_pos.xz);
    var base: vec3<f32>;
    if      st == 0u { base = vec3<f32>(0.97, 0.97, 1.0); }
    else if st == 1u { base = vec3<f32>(0.73, 0.73, 0.82); }
    else if st == 2u { base = vec3<f32>(0.60, 0.67, 0.47); }
    else if st == 3u { base = vec3<f32>(0.27, 0.55, 0.20); }
    else if st == 4u { base = vec3<f32>(0.53, 0.70, 0.25); }
    else if st == 5u { base = vec3<f32>(0.82, 0.77, 0.55); }
    else if st == 6u { base = vec3<f32>(0.45, 0.45, 0.50); }
    else if st == 7u { base = vec3<f32>(0.25, 0.35, 0.20); }
    else             { base = vec3<f32>(0.18, 0.25, 0.55); }

    if n.y < 0.4 { base = vec3<f32>(0.35, 0.35, 0.40); }
    return vec4<f32>(base * diffuse, 1.0);
#endif
}
