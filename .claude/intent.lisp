;; Atom Architecture Descriptor — intent.lisp
;; auto-generated 2026-06-20, ~3.1KB, target compression 34:1

(intent atom
  (design-constraints
    (no-unwrap "use expect(reason) instead")
    (dependency-tier (stdlib workspace-dep bevy-ecosystem conditional-new))
    (gpu-mesh-boundary "visual only; never physics or pathfinding")
    (dual-impl "critical algorithms mirrored CPU+GPU: quadric.rs ↔ quadric.wgsl")
    (no-touch (".atom.project" "performance-critical code")))

  (pillar terrain
    (purpose "GPU Dual Contouring voxel terrain with Voronoi biome generation")
    (state-machine "None → LoadAssets → GenerateTerrainRegion → GenerateTerrainMesh")
    (system-chain "ChunkLoader → ApplyCSG → GenerateChunk" (run-if in_state Mesh))

    (component chunk-lifecycle
      (role "Chunk spawning, GPU compute dispatch, mesh readback")
      (states (TerrainChunkMeshingState Idle Meshing))
      (data-flow "observer → update_grid_chunks → load_msgs → GPU 4-pass → crossbeam → TerrainChunkVisual")
      (symbols
        (type TerrainChunkCoord (tuple i32 i32 i32))
        (type TerrainChunk "marker component")
        (type TerrainChunkMeshingState (enum Idle Meshing))
        (resource TerrainLoadedChunks (hashmap TerrainChunkCoord Entity))
        (message TerrainChunkLoadMsg (coord TerrainChunkCoord))
        (message TerrainChunkUnloadMsg (coord TerrainChunkCoord))))

    (component gpu-pipeline
      (role "4-pass DC mesh generation on GPU")
      (passes
        (1 voxel_vertices "evaluate density at (N+1)³ grid points")
        (2 voxel_cross_points "binary search 8 iters, find isosurface edge crossings")
        (3 compute_vertices "QEF vertex fitting — probabilistic quadric + minimizer")
        (4 compute_indices "detect edge sign changes, emit triangle indices"))
      (seam "N+1 overlap; deterministic QEF produces identical boundary vertices naturally")
      (no-lod "top-down isometric view, all chunks fixed max resolution")
      (symbols
        (type TerrainChunkInfo (chunk_min_location_size vec4f voxel_size f32 voxel_num u32 qef_threshold f32 qef_stddev f32))
        (type TerrainMapConfig (terrain_height f32 pixel_size f32 temperature_min f32 temperature_max f32 use_biome u32))
        (buffer voxel_vertex_values "f32[] (N+1)³")
        (buffer voxel_cross_points "VoxelEdgeCrossPoint[]")
        (buffer mesh_vertices "TerrainChunkVertexInfo[]")
        (buffer mesh_indices "u32[]")
        (type VoxelEdgeCrossPoint (cross_location vec4f normal_material_index vec4f))))

    (component biome
      (role "CPU Voronoi → GPU texture → density field sampling")
      (no-biome-path "use_biome=0 → pure 3-layer FBM noise height function")
      (symbols
        (type BiomeType (enum Ocean=0 Forest=1 Desert=2 Plains=3 Mountains=4 Swamp=5))
        (type IsosurfaceSide (enum Outside:true Inside:false))
        (fn get_terrain_noise (location vec3f) → f32)
        (fn get_terrain_noise_no_biome (location vec3f) → f32)))

    (component material
      (role "Triplanar/biplanar PBR rendering")
      (symbols
        (type TerrainMaterial "standard Bevy Material trait")
        (type TerrainMaterialUniform (lod u32 roughness f32 metallic f32 flags u32 reflectance f32))
        (type TerrainMaterialKey "bind group specialization key"))))

  (pillar rendering
    (purpose "GPU buffer abstractions, shader loading, debug shapes, cel shading")

    (component gpu-buffer (crate atom_render)
      (symbols
        (type SharedStorageBuffer<T> "GPU read-write storage")
        (type SharedStagedBuffer<T> "CPU→GPU staged upload")))

    (component shader-system (crate atom_shader_lib)
      (macro shaders_plugin! "auto-generate Plugin + Resource from shader paths")
      (module gpu_test "GPU numeric test suite for noise/QEF verification")
      (module open_simplex "CPU OpenSimplex 2D — test code, upgrade to production for CPU SDF")))

  (pillar skill-system
    (purpose "EffectGraph node-based ability system with LayerTag-driven state management")

    (component effect-graph (crate atom_ability)
      (symbols
        (type EffectGraph "node graph runtime")
        (type Blackboard "per-instance variable store")
        (type AbilityContext "execution context")
        (type StateSet "runtime state collection")))

    (component layertag (crate atom_layertag)
      (symbols
        (type LayerTag "dot-separated path: ability.stun.fire")
        (type LayerTagBuilder)
        (fn exact_match)
        (fn partial_match)
        (fn same_prefix)
        (type SingleLayerTagContainer "HashSet — dedup")
        (type CountLayerTagContainer "Vec — ref-counted"))))

  (pillar data
    (purpose "Luban Excel-driven data tables with async loading")

    (component tables (crate atom_datatables)
      (state-machine "Wait → Loading → Loaded")
      (symbols
        (type TableReader<T> "SystemParam for key-based row lookup")
        (type AllAssetBarrier "named async barrier registry")
        (type Tables "resource holding loaded table handles")
        (event TablesLoadedEvent))
      (dependency atom_utils))

    (component barrier (crate atom_utils)
      (symbols
        (type AssetBarrier "custom Future: Arc<AtomicUsize> + Mutex<Waker>")
        (type AllAssetBarrier "named barrier registry"))))

  (pillar support
    (component paths (crate atom_core)
      (symbols (fn root_path "finds .atom.project marker") (fn assets_path) (fn config_root_path))
      (no-touch ".atom.project"))

    (component math (crate atom_math)
      (symbols "barycentric coordinates, triangle rasterization"))

    (component renderdoc (crate atom_renderdoc)
      (bind "F12 capture launch"))

    (component cel-shader (crate atom_cel_shader)
      (symbols (type CelMaterial) (type BackFacingMaterial)))

    (component codegen (crates atom_cfg atom_macros atom_luban_lib)
      (role "Luban .NET code generation output; ByteBuf binary deserializer")))

  (build
    (entry "cargo run -p atom_terrain --example chunk_loader")
    (ci "just ci = check → clippy → bevy_lint → nextest")
    (shaders "assets/shaders/{terrain,quadric,noise,csg,cel_shader,...}/")
    (config "Excel → Luban → .bytes → TableReader<T>")))