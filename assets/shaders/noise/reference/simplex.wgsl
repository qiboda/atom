#define_import_path noise

// This is a modified wgsl version from https://github.com/ashima/webgl-noise/blob/master/src/noise2D.glsl
// 
// Author: Ian McEwan, Ashima Arts
// GitHub: https://github.com/ashima/webgl-noise
//         https://github.com/stegu/webgl-noise
// Original License:
//   MIT License
//   Copyright (C) 2011 Ashima Arts
//   Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//   The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
// This modification is also licensed under the MIT License.
//
// MIT License
// Copyright © 2023 Zaron Chen (Ported to WGSL)
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

fn mod289v2f(x: vec2f) -> vec2f { return x - floor(x / 289.0) * 289.0; }
fn mod289v3f(x: vec3f) -> vec3f { return x - floor(x / 289.0) * 289.0; }
fn permute289v3f(x: vec3f) -> vec3f { return mod289v3f(((x * 34.0) + 10.0) * x); }

fn simplex_noise_2d(v: vec2f) -> f32 {
    let C = vec4f(0.211324865405187,
        0.366025403784439,
        -0.577350269189626,
        0.024390243902439);

    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    let i1 = select(vec2f(0.0, 1.0), vec2f(1.0, 0.0), (x0.x > x0.y));

    var x12 = x0.xyxy + C.xxzz;
    x12 = vec4f(x12.xy - i1, x12.zw);

    i = mod289v2f(i);
    let p = permute289v3f(permute289v3f(i.y + vec3f(0.0, i1.y, 1.0)) + i.x + vec3f(0.0, i1.x, 1.0));

    var m = max(0.5 - vec3f(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3f(0.0));
    m = m * m;
    m = m * m;

    let x = 2.0 * fract(p * C.www) - 1.0;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    let g = vec3f((a0.x * x0.x + h.x * x0.y), (a0.yz * x12.xz + h.yz * x12.yw));
    return 130.0 * dot(m, g);
}


// This is a modified wgsl version from https://github.com/ashima/webgl-noise/blob/master/src/noise3D.glsl
// 
// Author: Ian McEwan, Ashima Arts
// GitHub: https://github.com/ashima/webgl-noise
//         https://github.com/stegu/webgl-noise
// Original License:
//   MIT License
//   Copyright (C) 2011 Ashima Arts
//   Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//   The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
// This modification is also licensed under the MIT License.
//
// MIT License
// Copyright © 2023 Zaron Chen (Ported to WGSL)
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

fn mod289v3f(x: vec3f) -> vec3f { return x - floor(x / 289.0) * 289.0; }
fn mod289v4f(x: vec4f) -> vec4f { return x - floor(x / 289.0) * 289.0; }
fn permute289v4f(x: vec4f) -> vec4f { return mod289v4f(((x * 34.0) + 10.0) * x); }
fn taylorInvSqrtv4f(r: vec4f) -> vec4f { return 1.79284291400159 - 0.85373472095314 * r; }

fn simplex_noise_3d(v: vec3f) -> f32 {
    let C = vec2f(1. / 6., 1. / 3.);
    let D = vec4f(0., .5, 1., 2.);

    var i = floor(v + dot(v, C.yyy));
    var x0 = v - i + dot(i, C.xxx);

    var g = step(x0.yzx, x0.xyz);
    var l = 1.0 - g;
    var i1 = min(g.xyz, l.zxy);
    var i2 = max(g.xyz, l.zxy);

    var x1 = x0 - i1 + C.xxx;
    var x2 = x0 - i2 + C.yyy;
    var x3 = x0 - D.yyy;

    i = mod289v3f(i);
    var p = permute289v4f(permute289v4f(permute289v4f(
        i.z + vec4(0.0, i1.z, i2.z, 1.0)
    ) + i.y + vec4(0.0, i1.y, i2.y, 1.0)) + i.x + vec4(0.0, i1.x, i2.x, 1.0));

    var n_ = 0.142857142857;
    var ns = n_ * D.wyz - D.xzx;

    var j = p - 49.0 * floor(p * ns.z * ns.z);

    var x_ = floor(j * ns.z);
    var y_ = floor(j - 7.0 * x_);

    var x = x_ * ns.x + ns.yyyy;
    var y = y_ * ns.x + ns.yyyy;
    var h = 1.0 - abs(x) - abs(y);

    var b0 = vec4f(x.xy, y.xy);
    var b1 = vec4f(x.zw, y.zw);

    var s0 = floor(b0) * 2.0 + 1.0;
    var s1 = floor(b1) * 2.0 + 1.0;
    var sh = -step(h, vec4(0.0));

    var a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    var a1 = b1.xzyw + s1.xzyw * sh.zzww;

    var p0 = vec3f(a0.xy, h.x);
    var p1 = vec3f(a0.zw, h.y);
    var p2 = vec3f(a1.xy, h.z);
    var p3 = vec3f(a1.zw, h.w);

    var norm = taylorInvSqrtv4f(vec4f(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;

    var m = max(0.5 - vec4f(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3)), vec4f(0.0));
    m = m * m;

    return 105.0 * dot(m * m, vec4f(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3)));
}


// This is a modified wgsl version from https://github.com/ashima/webgl-noise/blob/master/src/noise3Dgrad.glsl
// 
// Author: Ian McEwan, Ashima Arts
// GitHub: https://github.com/ashima/webgl-noise
//         https://github.com/stegu/webgl-noise
// Original License:
//   MIT License
//   Copyright (C) 2011 Ashima Arts
//   Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//   The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
// This modification is also licensed under the MIT License.
//
// MIT License
// Copyright © 2023 Zaron Chen (Ported to WGSL)
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

fn mod289v3f(x: vec3f) -> vec3f { return x - floor(x / 289.0) * 289.0; }
fn mod289v4f(x: vec4f) -> vec4f { return x - floor(x / 289.0) * 289.0; }
fn permute289v4f(x: vec4f) -> vec4f { return mod289v4f(((x * 34.0) + 10.0) * x); }
fn taylorInvSqrtv4f(r: vec4f) -> vec4f { return 1.79284291400159 - 0.85373472095314 * r; }

fn simplex_noise_3d_grad(v: vec3f) -> f32 {
    let C = vec2f(1. / 6., 1. / 3.);
    let D = vec4f(0., .5, 1., 2.);

    var i = floor(v + dot(v, C.yyy));
    var x0 = v - i + dot(i, C.xxx);

    var g = step(x0.yzx, x0.xyz);
    var l = 1.0 - g;
    var i1 = min(g.xyz, l.zxy);
    var i2 = max(g.xyz, l.zxy);

    var x1 = x0 - i1 + C.xxx;
    var x2 = x0 - i2 + C.yyy;
    var x3 = x0 - D.yyy;

    i = mod289v3f(i);
    var p = permute289v4f(permute289v4f(permute289v4f(
        i.z + vec4(0.0, i1.z, i2.z, 1.0)
    ) + i.y + vec4(0.0, i1.y, i2.y, 1.0)) + i.x + vec4(0.0, i1.x, i2.x, 1.0));

    var n_ = 0.142857142857;
    var ns = n_ * D.wyz - D.xzx;

    var j = p - 49.0 * floor(p * ns.z * ns.z);

    var x_ = floor(j * ns.z);
    var y_ = floor(j - 7.0 * x_);

    var x = x_ * ns.x + ns.yyyy;
    var y = y_ * ns.x + ns.yyyy;
    var h = 1.0 - abs(x) - abs(y);

    var b0 = vec4f(x.xy, y.xy);
    var b1 = vec4f(x.zw, y.zw);

    var s0 = floor(b0) * 2.0 + 1.0;
    var s1 = floor(b1) * 2.0 + 1.0;
    var sh = -step(h, vec4(0.0));

    var a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    var a1 = b1.xzyw + s1.xzyw * sh.zzww;

    var p0 = vec3f(a0.xy, h.x);
    var p1 = vec3f(a0.zw, h.y);
    var p2 = vec3f(a1.xy, h.z);
    var p3 = vec3f(a1.zw, h.w);

    var norm = taylorInvSqrtv4f(vec4f(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;

    var m = max(0.5 - vec4f(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3)), vec4f(0.0));
    var m2 = m * m;
    var m4 = m2 * m2;
    var pdotx = vec4f(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3));

    var temp = m2 * m * pdotx;
    var gradient = -8.0 * (temp.x * x0 + temp.y * x1 + temp.z * x2 + temp.w * x3);
    gradient += m4.x * p0 + m4.y * p1 + m4.z * p2 + m4.w * p3;
    gradient *= 105.0;

    return 105.0 * dot(m4, pdotx);
}

// This is a modified wgsl version from https://github.com/ashima/webgl-noise/blob/master/src/noise4D.glsl
// 
// Author: Ian McEwan, Ashima Arts
// GitHub: https://github.com/ashima/webgl-noise
//         https://github.com/stegu/webgl-noise
// Original License:
//   MIT License
//   Copyright (C) 2011 Ashima Arts
//   Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//   The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
// This modification is also licensed under the MIT License.
//
// MIT License
// Copyright © 2023 Zaron Chen (Ported to WGSL)
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

fn mod289f32(x: f32) -> f32 { return x - floor(x / 289.0) * 289.0; }
fn mod289v4f(x: vec4f) -> vec4f { return x - floor(x / 289.0) * 289.0; }
fn permute289f32(x: f32) -> f32 { return mod289f32(((x * 34.0) + 10.0) * x); }
fn permute289v4f(x: vec4f) -> vec4f { return mod289v4f(((x * 34.0) + 10.0) * x); }
fn taylorInvSqrtf32(r: f32) -> f32 { return 1.79284291400159 - 0.85373472095314 * r; }
fn taylorInvSqrtv4f(r: vec4f) -> vec4f { return 1.79284291400159 - 0.85373472095314 * r; }

fn grad4(j: f32, ip: vec4f) -> vec4f {
    let ones = vec4f(1.0, 1.0, 1.0, -1.0);
    var p: vec4f;
    var s: vec4f;
    p = vec4f(floor(fract(vec3f(j) * ip.xyz) * 7.0) * ip.z - 1.0, p.w);
    p.w = 1.5 - dot(abs(p.xyz), ones.xyz);
    s = vec4f(p < vec4f(0.0));
    p = vec4f(p.xyz + (s.xyz * 2.0 - 1.0) * s.www, p.w);
    return p;
}

fn simplex_noise_4d(v: vec4f) -> f32 {
    let F4 = 0.309016994374947451;
    let C = vec4f(0.138196601125011,
        0.276393202250021,
        0.414589803375032,
        -0.447213595499958);

    var i = floor(v + dot(v, vec4f(F4)));
    let x0 = v - i + dot(i, C.xxxx);

    var i0: vec4f;
    let isX = step(x0.yzw, x0.xxx);
    let isYZ = step(x0.zww, x0.yyz);

    i0.x = isX.x + isX.y + isX.z;
    i0 = vec4f(i0.x, (1.0 - isX));

    i0.y += isYZ.x + isYZ.y;
    i0.z += 1.0 - isYZ.x;
    i0.w += 1.0 - isYZ.y;
    i0.z += isYZ.z;
    i0.w += 1.0 - isYZ.z;

    let i3 = clamp(i0, vec4f(0.0), vec4f(1.0));
    let i2 = clamp(i0 - 1.0, vec4f(0.0), vec4f(1.0));
    let i1 = clamp(i0 - 2.0, vec4f(0.0), vec4f(1.0));

    let x1 = x0 - i1 + C.xxxx;
    let x2 = x0 - i2 + C.yyyy;
    let x3 = x0 - i3 + C.zzzz;
    let x4 = x0 + C.wwww;

    i = mod289v4f(i);
    let j0 = permute289f32(permute289f32(permute289f32(permute289f32(i.w) + i.z) + i.y) + i.x);
    let j1 = permute289v4f(permute289v4f(permute289v4f(permute289v4f(
        i.w + vec4(i1.w, i2.w, i3.w, 1.0)
    ) + i.z + vec4(i1.z, i2.z, i3.z, 1.0)) + i.y + vec4(i1.y, i2.y, i3.y, 1.0)) + i.x + vec4(i1.x, i2.x, i3.x, 1.0));

    let ip = vec4(1.0 / 294.0, 1.0 / 49.0, 1.0 / 7.0, 0.0);

    var p0 = grad4(j0, ip);
    var p1 = grad4(j1.x, ip);
    var p2 = grad4(j1.y, ip);
    var p3 = grad4(j1.z, ip);
    var p4 = grad4(j1.w, ip);

    let norm = taylorInvSqrtv4f(vec4f(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;
    p4 *= taylorInvSqrtf32(dot(p4, p4));

    var m0 = max(0.6 - vec3f(dot(x0, x0), dot(x1, x1), dot(x2, x2)), vec3f(0.0));
    var m1 = max(0.6 - vec2f(dot(x3, x3), dot(x4, x4)), vec2f(0.0));
    m0 = m0 * m0;
    m1 = m1 * m1;
    return 49.0 * (dot(m0 * m0, vec3f(dot(p0, x0), dot(p1, x1), dot(p2, x2))) + dot(m1 * m1, vec2f(dot(p3, x3), dot(p4, x4))));
}