// Pass 5: 填充 Indirect Draw Command — 从 atomics 计数构造 DrawIndexedIndirect。
// 必须在 index_build (pass 4) 之后执行，读取最终的 counters。

struct GlobalUniforms {
    grid_min: vec3<f32>,
    pad0: u32,
    voxel_size: f32,
    grid_size: u32,
    pad1: vec2<u32>,
}

@group(0) @binding(0) var<uniform> info: GlobalUniforms;
@group(0) @binding(5) var<storage, read_write> counters: array<u32>;
@group(0) @binding(7) var<storage, read_write> indirect: array<u32>;

// DrawIndexedIndirect layout (24 bytes):
//   indirect[0] = index_count
//   indirect[1] = instance_count (always 1)
//   indirect[2] = first_index   (always 0)
//   indirect[3] = vertex_offset (always 0, i32)
//   indirect[4] = first_instance (always 0)
//   indirect[5] = pad (0)

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    // 单线程执行
    indirect[0] = counters[1]; // index_count
    indirect[1] = 1u;          // instance_count
    indirect[2] = 0u;          // first_index
    indirect[3] = 0u;          // vertex_offset
    indirect[4] = 0u;          // first_instance
    indirect[5] = 0u;          // pad
}
