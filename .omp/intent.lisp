;; Atom Architecture Descriptor — intent.lisp
;; 仅记录 rust-doc 无法表达的高层架构：数据流、管线、约束。
;; 符号级导航 → cargo doc --open

(intent atom
  (workspace "crates/atom_terrain + agent/ (TypeScript sidecar)")

  (data-flow "observer → update_grid_chunks → ChunkLoadMsg → handle_load_requests → TerrainChunksToProcess (ExtractResource) → terrain_compute_system (Render) → crossbeam → handle_mesh_data → Mesh3D"
    (sync-mechanism "TerrainChunksToProcess: ExtractResource clones main→render each frame; idempotent allocation check prevents double-init")
    (mesh-readback "GPU sparse vertices (voxel index) → CPU compact + remap → Mesh"))

  (gpu-pipeline "4-pass Dual Contouring on GPU"
    (pass 1 "voxel_vertices.wgsl — density = y - height_at(x,z), value noise FBM")
    (pass 2 "voxel_cross_points.wgsl — edge crossing binary search, central-diff normals")
    (pass 3 "main_mesh_compute_vertices.wgsl — QEF 3×3 Cramer's rule per voxel")
    (pass 4 "main_mesh_compute_indices.wgsl — DC quad (4 voxels per crossed edge → 2 triangles)")
    (buffer-layout "6 bindings: uniform(0) + storage(1-5). Fixed slot: vertices[voxel_idx], indices[6×voxel_idx]")
    (no-atomics "atom 类型匹配问题规避; sparse write + CPU compact")
    (seam "N+1 overlap grid; QEF deterministic across chunk boundaries"))

  (density-field "y - height_at(x,z), positive=air, negative=solid"
    (noise "value noise 3-octave FBM (MVP); replace with OpenSimplex when biome phase begins"))

  (agent-sidecar "TypeScript + BRP remote scripting"
    (runtime "npx tsx agent/src/index.ts — Bevy spawns as child process")
    (protocol "Bevy Remote Protocol (BRP) — HTTP JSON-RPC at 127.0.0.1:15702")
    (lifecycle "start_agent (Startup) spawns → AgentProcess(Child) resource → Drop kills on App exit")
    (data-flow "Agent → brp('world.query', Player+Transform) → player position → brp('world.spawn_entity') → NPC entity")
    (npc-visualize "Agent spawns entity with Name('NPC') + Transform only; Bevy decorate_agent_entities system adds Cube mesh + red Material")
    (constraints
      (brp-no-handles "world.spawn_entity cannot create Mesh/Material handles → Bevy-side system patches")
      (poll-delay "2s polling; not real-time → suitable for strategic/logic layer, not combat")))

  (constraints
    (no-unwrap "expect(\"reason\") required by clippy")
    (no-async-runtime "std-only futures; Bevy-compatible")
    (gpu-mesh-boundary "visual only; SDF query for collisions (future)")
    (no-touch ".atom.project"))

  (build
    (entry "cargo run -p atom_terrain --example top_down_game --release")
    (agent-entry "npx tsx agent/src/index.ts (auto-launched by Bevy)")
    (ci "just ci = check → clippy → bevy_lint → nextest")))
