#define_import_path noise::celular

// This is a modified wgsl version from https://github.com/stegu/webgl-noise/blob/master/src/cellular2D.glsl
// 
// Author: Stefan Gustavson (stefan.gustavson@liu.se)
// GitHub: https://github.com/stegu/webgl-noise
// Original License:
//   MIT License
//   Copyright (C) 2011 Stefan Gustavson
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
fn mod289v2f(x: vec2f) -> vec2f { return x - floor(x / 289.0) * 289.0; }
fn mod7v3f(x: vec3f) -> vec3f { return x - floor(x / 6.999999) * 6.999999; }
fn permute289v3f(x: vec3f) -> vec3f { return mod289v3f((34.0 * x + 10.0) * x); }

fn cellular_noise_2d(P: vec2f) -> vec2f {
    let K = 0.142857142857;
    let Ko = 0.428571428571;
    let jitter = 1.0;
    let Pi = mod289v2f(floor(P));
    let Pf = fract(P);
    let Oi = vec3f(-1.0, 0.0, 1.0);
    let Of = vec3f(-0.5, 0.5, 1.5);
    let px = permute289v3f(Pi.x + Oi);
    var p = permute289v3f(px.x + Pi.y + Oi);
    var ox = vec3f(fract(p * K) - Ko);
    var oy = mod7v3f(floor(p * K)) * K - Ko;
    var dx = vec3f(Pf.x + 0.5 + jitter * ox);
    var dy = vec3f(Pf.y - Of + jitter * oy);
    var d1 = vec3f(dx * dx + dy * dy);
    p = permute289v3f(px.y + Pi.y + Oi);
    ox = fract(p * K) - Ko;
    oy = mod7v3f(floor(p * K)) * K - Ko;
    dx = Pf.x - 0.5 + jitter * ox;
    dy = Pf.y - Of + jitter * oy;
    var d2 = vec3f(dx * dx + dy * dy);
    p = permute289v3f(px.z + Pi.y + Oi);
    ox = fract(p * K) - Ko;
    oy = mod7v3f(floor(p * K)) * K - Ko;
    dx = Pf.x - 1.5 + jitter * ox;
    dy = Pf.y - Of + jitter * oy;
    let d3 = vec3f(dx * dx + dy * dy);
    let d1a = min(d1, d2);
    d2 = max(d1, d2);
    d2 = min(d2, d3);
    d1 = min(d1a, d2);
    d2 = max(d1a, d2);
    d1 = select(d1.yxz, d1, (d1.x < d1.y));
    d1 = select(d1.zyx, d1, (d1.x < d1.z));
    d1 = vec3f(d1.x, min(d1.yz, d2.yz));
    d1 = vec3f(d1.x, min(d1.y, d1.z), d1.z);
    d1 = vec3f(d1.x, min(d1.y, d2.x), d1.z);
    return sqrt(d1.xy);
}


// This is a modified wgsl version from https://github.com/stegu/webgl-noise/blob/master/src/cellular2x2.glsl
// 
// Author: Stefan Gustavson (stefan.gustavson@liu.se)
// GitHub: https://github.com/stegu/webgl-noise
// Original License:
//   MIT License
//   Copyright (C) 2011 Stefan Gustavson
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
fn mod289v4f(x: vec4f) -> vec4f { return x - floor(x / 289.0) * 289.0; }
fn mod7v4f(x: vec4f) -> vec4f { return x - floor(x / 6.999999) * 6.999999; }
fn permute289v4f(x: vec4f) -> vec4f { return mod289v4f((34.0 * x + 10.0) * x); }

fn cellular_noise_2x2d(P: vec2f) -> vec2f {
    let K = 0.142857142857;
    let K2 = 0.0714285714285;
    let jitter = 0.8;
    let Pi = mod289v2f(floor(P));
    let Pf = fract(P);
    let Pfx = Pf.x + vec4f(-0.5, -1.5, -0.5, -1.5);
    let Pfy = Pf.y + vec4f(-0.5, -0.5, -1.5, -1.5);
    var p = permute289v4f(Pi.x + vec4f(0.0, 1.0, 0.0, 1.0));
    p = permute289v4f(p + Pi.y + vec4f(0.0, 0.0, 1.0, 1.0));
    let ox = mod7v4f(p) * K + K2;
    let oy = mod7v4f(floor(p * K)) * K + K2;
    let dx = Pfx + jitter * ox;
    let dy = Pfy + jitter * oy;
    var d = dx * dx + dy * dy;

    d = select(d.yxzw, d.xyzw, (d.x < d.y));
    d = select(d.zyxw, d.xyzw, (d.x < d.z));
    d = select(d.wyzx, d.xyzw, (d.x < d.w));
    d.y = min(d.y, d.z);
    d.y = min(d.y, d.w);
    return sqrt(d.xy);
}



// This is a modified wgsl version from https://github.com/stegu/webgl-noise/blob/master/src/cellular2x2x2.glsl
// 
// Author: Stefan Gustavson (stefan.gustavson@liu.se)
// GitHub: https://github.com/stegu/webgl-noise
// Original License:
//   MIT License
//   Copyright (C) 2011 Stefan Gustavson
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
fn mod7v4f(x: vec4f) -> vec4f { return x - floor(x / 6.999999) * 6.999999; }
fn permute289v3f(x: vec3f) -> vec3f { return mod289v3f((34.0 * x + 10.0) * x); }
fn permute289v4f(x: vec4f) -> vec4f { return mod289v4f((34.0 * x + 10.0) * x); }

fn cellular_noise_2x2x2d(P: vec3f) -> vec2f {
    let K = 0.142857142857;
    let Ko = 0.428571428571;
    let K2 = 0.020408163265306;
    let Kz = 0.166666666667;
    let Kzo = 0.416666666667;
    let jitter = 0.8;
    let Pi = mod289v3f(floor(P));
    let Pf = fract(P);
    let Pfx = Pf.x + vec4f(0.0, -1.0, 0.0, -1.0);
    let Pfy = Pf.y + vec4f(0.0, 0.0, -1.0, -1.0);
    var p = permute289v4f(Pi.x + vec4f(0.0, 1.0, 0.0, 1.0));
    p = permute289v4f(p + Pi.y + vec4f(0.0, 0.0, 1.0, 1.0));
    let p1 = permute289v4f(p + Pi.z);
    let p2 = permute289v4f(p + Pi.z + vec4(1.0));
    let ox1 = fract(p1 * K) - Ko;
    let oy1 = mod7v4f(floor(p1 * K)) * K - Ko;
    let oz1 = floor(p1 * K2) * Kz - Kzo;
    let ox2 = fract(p2 * K) - Ko;
    let oy2 = mod7v4f(floor(p2 * K)) * K - Ko;
    let oz2 = floor(p2 * K2) * Kz - Kzo;
    let dx1 = Pfx + jitter * ox1;
    let dy1 = Pfy + jitter * oy1;
    let dz1 = Pf.z + jitter * oz1;
    let dx2 = Pfx + jitter * ox2;
    let dy2 = Pfy + jitter * oy2;
    let dz2 = Pf.z - 1.0 + jitter * oz2;
    let d1 = dx1 * dx1 + dy1 * dy1 + dz1 * dz1;
    var d2 = dx2 * dx2 + dy2 * dy2 + dz2 * dz2;

    var d = min(d1, d2);
    d2 = max(d1, d2);
    d = select(d.yxzw, d.xyzw, (d.x < d.y));
    d = select(d.zyxw, d.xyzw, (d.x < d.z));
    d = select(d.wyzx, d.xyzw, (d.x < d.w));
    d = vec4f(d.x, min(d.yzw, d2.yzw));
    d.y = min(d.y, d.z);
    d.y = min(d.y, d.w);
    d.y = min(d.y, d2.x);
    return sqrt(d.xy);
}


// This is a modified wgsl version from https://github.com/stegu/webgl-noise/blob/master/src/cellular3D.glsl
// 
// Author: Stefan Gustavson (stefan.gustavson@liu.se)
// GitHub: https://github.com/stegu/webgl-noise
// Original License:
//   MIT License
//   Copyright (C) 2011 Stefan Gustavson
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
fn mod7v3f(x: vec3f) -> vec3f { return x - floor(x / 6.999999) * 6.999999; }
fn permute289v3f(x: vec3f) -> vec3f { return mod289v3f((34.0 * x + 10.0) * x); }

fn cellular_noise_3d(P: vec3f) -> vec2f {
    let K = 0.142857142857;
    let Ko = 0.428571428571;
    let K2 = 0.020408163265306;
    let Kz = 0.166666666667;
    let Kzo = 0.416666666667;
    let jitter = 1.0;

    let Pi = mod289v3f(floor(P));
    let Pf = fract(P) - 0.5;

    let Pfx = Pf.x + vec3f(1.0, 0.0, -1.0);
    let Pfy = Pf.y + vec3f(1.0, 0.0, -1.0);
    let Pfz = Pf.z + vec3f(1.0, 0.0, -1.0);

    let p = permute289v3f(Pi.x + vec3(-1.0, 0.0, 1.0));
    let p1 = permute289v3f(p + Pi.y - 1.0);
    let p2 = permute289v3f(p + Pi.y);
    let p3 = permute289v3f(p + Pi.y + 1.0);

    let p11 = permute289v3f(p1 + Pi.z - 1.0);
    let p12 = permute289v3f(p1 + Pi.z);
    let p13 = permute289v3f(p1 + Pi.z + 1.0);

    let p21 = permute289v3f(p2 + Pi.z - 1.0);
    let p22 = permute289v3f(p2 + Pi.z);
    let p23 = permute289v3f(p2 + Pi.z + 1.0);

    let p31 = permute289v3f(p3 + Pi.z - 1.0);
    let p32 = permute289v3f(p3 + Pi.z);
    let p33 = permute289v3f(p3 + Pi.z + 1.0);

    let ox11 = fract(p11 * K) - Ko;
    let oy11 = mod7v3f(floor(p11 * K)) * K - Ko;
    let oz11 = floor(p11 * K2) * Kz - Kzo;

    let ox12 = fract(p12 * K) - Ko;
    let oy12 = mod7v3f(floor(p12 * K)) * K - Ko;
    let oz12 = floor(p12 * K2) * Kz - Kzo;

    let ox13 = fract(p13 * K) - Ko;
    let oy13 = mod7v3f(floor(p13 * K)) * K - Ko;
    let oz13 = floor(p13 * K2) * Kz - Kzo;

    let ox21 = fract(p21 * K) - Ko;
    let oy21 = mod7v3f(floor(p21 * K)) * K - Ko;
    let oz21 = floor(p21 * K2) * Kz - Kzo;

    let ox22 = fract(p22 * K) - Ko;
    let oy22 = mod7v3f(floor(p22 * K)) * K - Ko;
    let oz22 = floor(p22 * K2) * Kz - Kzo;

    let ox23 = fract(p23 * K) - Ko;
    let oy23 = mod7v3f(floor(p23 * K)) * K - Ko;
    let oz23 = floor(p23 * K2) * Kz - Kzo;

    let ox31 = fract(p31 * K) - Ko;
    let oy31 = mod7v3f(floor(p31 * K)) * K - Ko;
    let oz31 = floor(p31 * K2) * Kz - Kzo;

    let ox32 = fract(p32 * K) - Ko;
    let oy32 = mod7v3f(floor(p32 * K)) * K - Ko;
    let oz32 = floor(p32 * K2) * Kz - Kzo;

    let ox33 = fract(p33 * K) - Ko;
    let oy33 = mod7v3f(floor(p33 * K)) * K - Ko;
    let oz33 = floor(p33 * K2) * Kz - Kzo;

    let dx11 = Pfx + jitter * ox11;
    let dy11 = Pfy.x + jitter * oy11;
    let dz11 = Pfz.x + jitter * oz11;

    let dx12 = Pfx + jitter * ox12;
    let dy12 = Pfy.x + jitter * oy12;
    let dz12 = Pfz.y + jitter * oz12;

    let dx13 = Pfx + jitter * ox13;
    let dy13 = Pfy.x + jitter * oy13;
    let dz13 = Pfz.z + jitter * oz13;

    let dx21 = Pfx + jitter * ox21;
    let dy21 = Pfy.y + jitter * oy21;
    let dz21 = Pfz.x + jitter * oz21;

    let dx22 = Pfx + jitter * ox22;
    let dy22 = Pfy.y + jitter * oy22;
    let dz22 = Pfz.y + jitter * oz22;

    let dx23 = Pfx + jitter * ox23;
    let dy23 = Pfy.y + jitter * oy23;
    let dz23 = Pfz.z + jitter * oz23;

    let dx31 = Pfx + jitter * ox31;
    let dy31 = Pfy.z + jitter * oy31;
    let dz31 = Pfz.x + jitter * oz31;

    let dx32 = Pfx + jitter * ox32;
    let dy32 = Pfy.z + jitter * oy32;
    let dz32 = Pfz.y + jitter * oz32;

    let dx33 = Pfx + jitter * ox33;
    let dy33 = Pfy.z + jitter * oy33;
    let dz33 = Pfz.z + jitter * oz33;

    var d11 = dx11 * dx11 + dy11 * dy11 + dz11 * dz11;
    var d12 = dx12 * dx12 + dy12 * dy12 + dz12 * dz12;
    var d13 = dx13 * dx13 + dy13 * dy13 + dz13 * dz13;
    var d21 = dx21 * dx21 + dy21 * dy21 + dz21 * dz21;
    var d22 = dx22 * dx22 + dy22 * dy22 + dz22 * dz22;
    var d23 = dx23 * dx23 + dy23 * dy23 + dz23 * dz23;
    var d31 = dx31 * dx31 + dy31 * dy31 + dz31 * dz31;
    var d32 = dx32 * dx32 + dy32 * dy32 + dz32 * dz32;
    var d33 = dx33 * dx33 + dy33 * dy33 + dz33 * dz33;

    let d1a = min(d11, d12);
    d12 = max(d11, d12);
    d11 = min(d1a, d13);
    d13 = max(d1a, d13);
    d12 = min(d12, d13);
    let d2a = min(d21, d22);
    d22 = max(d21, d22);
    d21 = min(d2a, d23);
    d23 = max(d2a, d23);
    d22 = min(d22, d23);
    let d3a = min(d31, d32);
    d32 = max(d31, d32);
    d31 = min(d3a, d33);
    d33 = max(d3a, d33);
    d32 = min(d32, d33);
    let da = min(d11, d21);
    d21 = max(d11, d21);
    d11 = min(da, d31);
    d31 = max(da, d31);
    d11 = select(d11.yxz, d11, (d11.x < d11.y));
    d11 = select(d11.zyx, d11, (d11.x < d11.z));
    d12 = min(d12, d21);
    d12 = min(d12, d22);
    d12 = min(d12, d31);
    d12 = min(d12, d32);
    d11 = vec3f(d11.x, min(d11.yz, d12.xy));
    d11.y = min(d11.y, d12.z);
    d11.y = min(d11.y, d11.z);
    return sqrt(d11.xy);
}