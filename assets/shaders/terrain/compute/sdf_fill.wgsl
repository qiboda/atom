// Pass 0: SDF Fill — density = y - height_at(x,z)
// Terrain: Voronoi cells + FBM noise + multi-island archipelago.
//
// Algorithm:
//   1. Grid of potential island centers (500m spacing, jittered)
//   2. Each grid cell: 30% chance of being an island (deterministic from seed)
//   3. Island radius: 150-500m (deterministic from seed)
//   4. FBM elevation × island falloff → continuous height 0-MAX_HEIGHT
//   5. Cell boundary cliffs where elevation difference > 3m
//   6. Whittaker biome diagram: (elevation, moisture) → 9 types
//   7. Ocean outside all islands

// ── Map constants ──
const MAP_SIZE: f32 = 4096.0;
const ISLAND_SPACING: f32 = 500.0;
const ISLAND_PROB: f32 = 0.3;
const ISLAND_MIN_R: f32 = 150.0;
const ISLAND_MAX_R: f32 = 500.0;

// ── Terrain constants ──
const CELL_SIZE: f32 = 30.0;

// ── Uniform ──
struct GlobalUniforms {
    grid_min: vec3<f32>,
    pad0: u32,
    voxel_size: f32,
    grid_size: u32,
    pad1: vec2<u32>,
    neighbor_mask: u32,
    pad3: u32,
    seed: u32,
    pad2: u32,
}

@group(0) @binding(0) var<uniform> info: GlobalUniforms;
@group(0) @binding(1) var<storage, read_write> density: array<f32>;

// ── Fast hash ──
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

fn fbm(x: f32, z: f32, octaves: u32, persistence: f32, seed: u32) -> f32 {
    var value = 0.0; var amplitude = 1.0; var frequency = 1.0; var max_val = 0.0;
    for (var i = 0u; i < octaves; i++) {
        value += noise_2d(x * frequency, z * frequency, seed + i * 97u) * amplitude;
        max_val += amplitude; amplitude *= persistence; frequency *= 2.0;
    }
    return value / max_val;
}

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

// ── Island placement (deterministic from seed) ──
struct IslandInfo {
    center: vec2<f32>,
    radius: f32,
    exists: bool,
}

fn island_at(gx: i32, gz: i32, seed: u32) -> IslandInfo {
    let h = hash_2d(gx, gz, seed ^ 0x15feed0u);
    let exists = hash_f32(h) < ISLAND_PROB;
    let ox = (hash_f32(h) - 0.5) * ISLAND_SPACING * 0.6;
    let oz = (hash_f32(hash_u32(h)) - 0.5) * ISLAND_SPACING * 0.6;
    let r = ISLAND_MIN_R + hash_f32(hash_u32(h ^ 0x55u)) * (ISLAND_MAX_R - ISLAND_MIN_R);
    var info: IslandInfo;
    info.center = vec2<f32>(f32(gx) * ISLAND_SPACING + ox, f32(gz) * ISLAND_SPACING + oz);
    info.radius = r;
    info.exists = exists;
    return info;
}

// ── Multi-island falloff ──
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
            let coast = fbm(pos.x * 0.003, pos.y * 0.003, 2u, 0.5, seed ^ 0x5a5a5a5au) * island.radius * 0.15;
            let d = max(d_raw + coast1 + coast2 + coast, 0.0);
            if d < island.radius * 1.1 {
                let f = 1.0 - smoothstep(island.radius * 0.5, island.radius, d);
            }
        }
    }
    return max_f;
}

// ── Voronoi cells ──
fn cell_seed_pos(cx: i32, cz: i32, world_seed: u32) -> vec2<f32> {
    let h = hash_2d(cx, cz, world_seed);
    let ox = (hash_f32(h) - 0.5) * CELL_SIZE * 0.7;
    let oz = (hash_f32(hash_u32(h)) - 0.5) * CELL_SIZE * 0.7;
    return vec2<f32>(f32(cx) * CELL_SIZE + ox, f32(cz) * CELL_SIZE + oz);
}


// ── Map edge ocean border ──
fn edge_falloff(pos: vec2<f32>) -> f32 {
    let half = 2048.0;
    let margin = 200.0;
    let d = max(abs(pos.x), abs(pos.y)) / (half - margin);
    return 1.0 - smoothstep(0.85, 1.0, d);
}

fn cell_elevation(cx: i32, cz: i32, seed: u32) -> f32 {
    let pos = cell_seed_pos(cx, cz, seed);
    let wx = pos.x * 0.004; let wz = pos.y * 0.004;
    let elev = (1.0 + fbm(wx, wz, 4u, 0.55, seed)) * 0.55;
    let falloff = island_falloff(pos, seed) * edge_falloff(pos);
    var e = elev * falloff;
    if e > 0.55 {
        let ridge = (e - 0.55) / 0.35;
        e = min(e + ridge * ridge * 0.25, 1.0);
    }
    return max(e, 0.0);
}

fn cell_temperature(cx: i32, cz: i32, seed: u32) -> f32 {
    let pos = cell_seed_pos(cx, cz, seed);
    let lat = (pos.y + 2048.0) / 4096.0;
    let noise = fbm(pos.x * 0.0004, pos.y * 0.0004, 3u, 0.55, seed ^ 0xdeedeedu);
    return clamp(lat * 0.55 + noise * 0.22 + 0.5, 0.0, 1.0);
}

fn cell_moisture(cx: i32, cz: i32, seed: u32) -> f32 {
    let pos = cell_seed_pos(cx, cz, seed);
    let wx = pos.x * 0.005 + 100.0; let wz = pos.y * 0.005 + 100.0;
    return fbm(wx, wz, 3u, 0.55, seed ^ 0xaaaaaaaau) * 0.5 + 0.5;
}

fn temperature_at(pos: vec2<f32>, seed: u32) -> f32 {
    let cx = i32(floor(pos.x / CELL_SIZE));
    let cz = i32(floor(pos.y / CELL_SIZE));
    return cell_temperature(cx, cz, seed);
}

fn moisture_at(pos: vec2<f32>, seed: u32) -> f32 {
    let cx = i32(floor(pos.x / CELL_SIZE));
    let cz = i32(floor(pos.y / CELL_SIZE));
    return cell_moisture(cx, cz, seed);
}

// ── Whittaker biome: (temperature, moisture, elevation) → type ──
fn biome_from_tme(temp: f32, moist: f32, elev: f32) -> u32 {
    if elev < 0.05 { return 8u; }  // Ocean
    if elev > 0.7  {
        if temp < 0.3 { return 0u; } else { return 6u; }  // Snow or Rock
    }
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

// ── Voronoi query ──
struct VoronoiResult {
    best_cx: i32, best_cz: i32,
    second_cx: i32, second_cz: i32,
    boundary_dist: f32,
}

fn voronoi_query(xz: vec2<f32>) -> VoronoiResult {
    let pn_x = fbm(xz.x * 0.03, xz.y * 0.03, 1u, 1.0, 42u + 200u) * 4.0;
    let pn_z = fbm(xz.x * 0.03 + 50.0, xz.y * 0.03 + 50.0, 1u, 1.0, 42u + 300u) * 4.0;
    let px = xz.x + pn_x; let pz = xz.y + pn_z;
    let cx = i32(floor(px / CELL_SIZE)); let cz = i32(floor(pz / CELL_SIZE));
    var best_d: f32 = 999999.0; var sec_d: f32 = 999999.0;
    var bcx: i32 = 0; var bcz: i32 = 0; var scx: i32 = 0; var scz: i32 = 0;
    for (var dx = -1; dx <= 1; dx++) {
        for (var dz = -1; dz <= 1; dz++) {
            let scx2 = cx + dx; let scz2 = cz + dz;
            let sp = cell_seed_pos(scx2, scz2, 42u);
            let d = distance(vec2<f32>(px, pz), sp);
            if d < best_d { sec_d = best_d; scx = bcx; scz = bcz; best_d = d; bcx = scx2; bcz = scz2; }
            else if d < sec_d { sec_d = d; scx = scx2; scz = scz2; }
        }
    }
    var r: VoronoiResult;
    r.best_cx = bcx; r.best_cz = bcz; r.second_cx = scx; r.second_cz = scz;
    r.boundary_dist = clamp((sec_d - best_d) * 0.5, 0.0, 10.0);
    return r;
}

// ── Biome base heights (m) ──
// Snow=0,Tundra=1,Taiga=2,Forest=3,Grassland=4,Desert=5,Rock=6,Swamp=7,Ocean=8
const BIOME_HEIGHT: array<f32, 9> = array<f32, 9>(
   -2.0,  // Ocean
    2.0,  // Desert
    3.0,  // Swamp
    5.0,  // Grassland
    8.0,  // Forest
   12.0,  // Taiga
   15.0,  // Tundra
   20.0,  // Snow
   18.0,  // Rock
);

// ── Height ──
fn height_at(xz: vec2<f32>) -> f32 {
    let pos = xz;
    let wx = pos.x * 0.004; let wy = pos.y * 0.004;
    let raw_elev = (1.0 + fbm(wx, wy, 4u, 0.55, 42u)) * 0.55
                 * island_falloff(pos, 42u) * edge_falloff(pos);
    let temp = temperature_at(pos, 42u);
    let moist = moisture_at(pos, 42u);
    let biome = biome_from_tme(temp, moist, max(raw_elev, 0.0));
    let base = BIOME_HEIGHT[biome];
    let detail = fbm(pos.x * 0.15, pos.y * 0.15, 2u, 0.5, 42u + 777u) * 0.5;
    let micro = (hash_f32(hash_2d(i32(floor(xz.x*5.0)), i32(floor(xz.y*5.0)), 42u+999u)) - 0.5) * 0.3;
    return base + detail + micro;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let n = info.grid_size + 2u;
    if gid.x >= n || gid.y >= n || gid.z >= n { return; }
    let idx = gid.x + gid.y * n + gid.z * n * n;
    let pos = info.grid_min + vec3<f32>(f32(gid.x), f32(gid.y), f32(gid.z)) * info.voxel_size;
    density[idx] = pos.y - height_at(pos.xz);
}
