#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import noise::open_simplex_seed::{open_simplex_2d_with_seed, open_simplex_3d_with_seed, open_simplex_4d_with_seed }
#import noise::fbm::{open_simplex_2d_fbm_with_seed, open_simplex_3d_fbm_with_seed, open_simplex_4d_fbm_with_seed }
#import random::xorshift_128::xorshift_128_with_seed
#import limit::numeric::U32_MAX
#import bevy_sprite::mesh2d_view_bindings::{ globals }

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let xy = mesh.uv;

    let time = globals.time;
    let loc = vec4f(xy.x, xy.y, time * 0.1, time * 0.1);
    let x = (open_simplex_4d_with_seed(loc * 100.0, 2302323u) + 1.0) / 2.0;
    let y = (open_simplex_4d_fbm_with_seed(loc * 100.0, 300002332u, 5u, 0.25, 1.1, 8.0) + 1.0) * 0.5;
    let z = (open_simplex_3d_fbm_with_seed(mesh.position.xyz * 1.0, 300002332u, 5u, 0.25, 1.1, 8.0) + 1.0) * 0.5;
    // let y = (open_simplex_2d_with_seed(mesh.uv * 10.0, 300002332u) + 1.0) / 2.0;
    // let z = (open_simplex_2d_with_seed(mesh.uv * 10.0, 387833293u) + 1.0) / 2.0;
    return vec4f(vec3f(y, y, z), 1.0);
    // return vec4f(mesh.uv, mesh.uv.x, 1.0);
}
