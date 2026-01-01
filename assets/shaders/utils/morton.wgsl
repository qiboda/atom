#define_import_path morton

// Expands a 10-bit integer into 30 bits
// by inserting 2 zeros after each bit.
fn expand_bits(a: u32) -> u32 {
    var v = a;
    v = (v * 0x00010001u) & 0xFF0000FFu;
    v = (v * 0x00000101u) & 0x0F00F00Fu;
    v = (v * 0x00000011u) & 0xC30C30C3u;
    v = (v * 0x00000005u) & 0x49249249u;
    return v;
}

fn morton_encode_3d(x: u32, y: u32, z: u32) -> u32 {
    let xx = expand_bits(x);
    let yy = expand_bits(y);
    let zz = expand_bits(z);
    return xx << 2u | yy << 1u | zz;
}