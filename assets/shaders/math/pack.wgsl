#define_import_path math::pack

fn pack4xU8(pack: vec4u) -> u32 {
    var packed = u32(0);
    packed += pack.x;
    packed += pack.y << 8;
    packed += pack.z << 16;
    packed += pack.w << 24;

    return packed;
}

// The builtin one didn't work in webgl.
// "'unpackUnorm4x8' : no matching overloaded function found"
// https://github.com/gfx-rs/naga/issues/2006
fn unpack4x8unorm(v: u32) -> vec4<f32> {
    return vec4(
        f32(v & 0xFFu),
        f32((v >> 8u) & 0xFFu),
        f32((v >> 16u) & 0xFFu),
        f32((v >> 24u) & 0xFFu)
    ) / 255.0;
}
