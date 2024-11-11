/// copy from https://iquilezles.org/articles/distfunctions/
#define_import_path csg::csg_shape

// 不是相加，而是相减
fn ndot(a: vec2f, b: vec2f) -> f32 {
    return a.x * b.x - a.y * b.y;
}

// 默认是精确的版本
// round: 平滑边缘的版本

// sd: signed distance

// 参数
// p: 检测位置
// 所有形状默认都是以原点为中心的

// 球形，精确版本
// s: 半径
fn sd_sphere(p: vec3f, s: f32) -> f32 {
    return length(p) - s;
}

// b: 边长
fn sd_box(p: vec3f, b: vec3f) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3f(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// b: 边长
// r: 圆角半径
fn sd_round_box(p: vec3f, b: vec3f, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0) - r;
}


// 矩形边框
// b: 边长
// e: 边框厚度
fn sd_box_frame(p: vec3f, b: vec3f, e: f32) -> f32 {
    let lp = abs(p) - b;
    let q = abs(lp + e) - e;
    return min(
        min(
            length(max(vec3f(lp.x, q.y, q.z), 0.0)) + min(max(lp.x, max(q.y, q.z)), 0.0),
            length(max(vec3f(q.x, lp.y, q.z), 0.0)) + min(max(q.x, max(lp.y, q.z)), 0.0)
        ),
        length(max(vec3f(q.x, q.y, lp.z), 0.0)) + min(max(q.x, max(q.y, lp.z)), 0.0)
    );
}

// 环面
fn sd_torus(p: vec3f, t: vec2f) -> f32 {
    let q = vec2f(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

// 一面截断的环面
fn sd_capped_torus(p: vec3f, sc: vec2f, ra: f32, rb: f32) -> f32 {
    var pp = p;
    pp.x = abs(pp.x);
    let k = select(length(pp.xy), dot(pp.xy, sc), sc.y * pp.x > sc.x * pp.y);
    return sqrt(dot(pp, pp) + ra * ra - 2.0 * ra * k) - rb;
}

// 单向拉伸的环面
fn sd_link(p: vec3f, le: f32, r1: f32, r2: f32) -> f32 {
    let q = vec3(p.x, max(abs(p.y) - le, 0.0), p.z);
    return length(vec2(length(q.xy) - r1, q.z)) - r2;
}

// 无限高度的圆柱
// 链接中名字：sd_cylinder
fn sd_infinite_cylinder(p: vec3f, c: vec3f) -> f32 {
    return length(p.xz - c.xy) - c.z;
}

// 圆锥
fn sd_cone(p: vec3f, c: vec2f, h: f32) -> f32 {
  // c is the sin/cos of the angle, h is height
  // Alternatively pass q instead of (c,h),
  // which is the point at the base in 2D
    let q = h * vec2(c.x / c.y, -1.0);

    let w = vec2(length(p.xz), p.y);
    let a = w - q * clamp(dot(w, q) / dot(q, q), 0.0, 1.0);
    let b = w - q * vec2(clamp(w.x / q.x, 0.0, 1.0), 1.0);
    let k = sign(q.y);
    let d = min(dot(a, a), dot(b, b));
    let s = max(k * (w.x * q.y - w.y * q.x), k * (w.y - q.y));
    return sqrt(d) * sign(s);
}

// 不精确的cone。
// 链接中名字：sdCone
fn sd_cone_2(p: vec3f, c: vec2f, h: f32) -> f32 {
    let q = length(p.xz);
    return max(dot(c.xy, vec2(q, p.y)), -h - p.y);
}

// Infinite Cone - exact
fn sd_infinite_cone(p: vec3f, c: vec2f) -> f32 {
    // c is the sin/cos of the angle
    let q = vec2(length(p.xz), -p.y);
    let d = length(q - c * max(dot(q, c), 0.0));
    return d * select(1.0, -1.0, (q.x * c.y - q.y * c.x < 0.0));
}

// 平面
fn sd_plane(p: vec3f, n: vec3f, h: f32) -> f32 {
  // n must be normalized
    return dot(p, n) + h;
}


// 六角棱柱
fn sd_hex_prism(p: vec3f, h: vec2f) -> f32 {
    let k: vec3f = vec3f(-0.8660254, 0.5, 0.57735);
    var pp = abs(p);
    pp.x = pp.x - 2.0 * min(dot(k.xy, pp.xy), 0.0) * k.x;
    pp.y = pp.y - 2.0 * min(dot(k.xy, pp.xy), 0.0) * k.y;
    let d = vec2(
        length(pp.xy - vec2(clamp(pp.x, -k.z * h.x, k.z * h.x), h.x)) * sign(pp.y - h.x),
        pp.z - h.y
    );
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

// 三角棱柱
fn sd_tri_prism(p: vec3f, h: vec2f) -> f32 {
    let q = abs(p);
    return max(q.z - h.y, max(q.x * 0.866025 + p.y * 0.5, -p.y) - h.x * 0.5);
}

// 胶囊体
fn sd_capsule(p: vec3f, a: vec3f, b: vec3f, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

// 垂直胶囊体
fn sd_vertical_capsule(p: vec3f, h: f32, r: f32) -> f32 {
    var pp = p;
    pp.y -= clamp(pp.y, 0.0, h);
    return length(pp) - r;
}

// 垂直带盖的圆柱体
fn sd_vertical_capped_cylinder(p: vec3f, h: f32, r: f32) -> f32 {
    let d = abs(vec2(length(p.xz), p.y)) - vec2(r, h);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

// 任意的带盖圆柱体
fn sd_capped_cylinder(p: vec3f, a: vec3f, b: vec3f, r: f32) -> f32 {
    let  ba = b - a;
    let  pa = p - a;
    let baba = dot(ba, ba);
    let paba = dot(pa, ba);
    let x = length(pa * baba - ba * paba) - r * baba;
    let y = abs(paba - baba * 0.5) - baba * 0.5;
    let x2 = x * x;
    let y2 = y * y * baba;

    let s1 = select(0.0, x2, x > 0.0);
    let s2 = select(0.0, y2, y > 0.0);
    let d = select(s1 + s2, -min(x2, y2), max(x, y) < 0.0);
    return sign(d) * sqrt(abs(d)) / baba;
}

// 圆角的圆柱体
fn sdRoundedCylinder(p: vec3f, ra: f32, rb: f32, h: f32) -> f32 {
    let d = vec2f(length(p.xz), -2.0 * ra + rb, abs(p.y) - h);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - rb;
}

// 截断的圆锥体。
// Capped Cone 
fn sd_vertical_capped_cone(p: vec3f, h: f32, r1: f32, r2: f32) {
    let q = vec2f(length(p.xz), p.y);
    let k1 = vec2f(r2, h);
    let k2 = vec2f(r2 - r1, 2.0 * h);
    let ca = vec2f(q.x - min(q.x, select(r2, r1, q.y < 0.0)), abs(q.y) - h);
    let cb = q - k1 + k2 * clamp(dot(k1 - q, k2) / dot(k2), 0.0, 1.0);
    let s = select(1.0, -1.0, cb.x < 0.0 && ca.y < 0.0);
    return s * sqrt(min(dot(ca), dot(cb)));
}

// 任意朝向 的截断圆锥体
fn sd_capped_cone(p: vec3f, a: vec3f, b: vec3f, ra: f32, rb: f32) -> f32 {
    let rba = rb - ra;
    let baba = dot(b - a, b - a);
    let papa = dot(p - a, p - a);
    let paba = dot(p - a, b - a) / baba;
    let x = sqrt(papa - paba * paba * baba);
    let cax = max(0.0, x - select(rb, ra, paba < 0.5));
    let cay = abs(paba, -0.5)-0.5;
    let k = rba * rba + baba;
    let f = clamp((rba * (x - ra) + paba * baba) / k, 0.0, 1.0);
    let cbx = x - ra - f * rba;
    let cby = paba - f;
    let s = select(1.0, -1.0, cbx < 0.0 && cay < 0.0);
    return s * sqrt(min(cax * cax + cay * cay * baba,
        cbx * cbx + cby * cby * baba));
}

// 立体角
fn sd_solid_angle(p: vec3f, c: vec2f, ra: f32) -> f32 {
  // c is the sin/cos of the angle
    let q = vec2f(length(p.xz), p.y);
    let l = length(q) - ra;
    let m = length(q - c * clamp(dot(q, c), 0.0, ra));
    return max(l, m * sign(c.y * q.x - c.x * q.y));
}

// 截断的球体
fn sd_cut_sphere(p: vec3f, r: f32, h: f32) -> f32 {
  // sampling independent computations (only depend on shape)
    let w = sqrt(r * r - h * h);

  // sampling dependant computations
    let q = vec2(length(p.xz), p.y);
    let s = max((h - r) * q.x * q.x + w * w * (h + r - 2.0 * q.y), h * q.x - w * q.y);

    let a = select(length(q - vec2(w, h)), h - q.y, q.x < w);
    return select(a, length(q) - r, s < 0.0);
}

// 阶段的中空的球体
fn sd_cut_hollow_sphere(p: vec3f, r: f32, h: f32, t: f32) -> f32 {
  // sampling independent computations (only depend on shape)
    let w = sqrt(r * r - h * h);
  
  // sampling dependant computations
    let q = vec2f(length(p.xz), p.y);
    return select(abs(length(q) - r), length(q - vec2(w, h)), h * q.x < w * q.y) - t;
}

// 死星：星球大战中的死星。
// 有点像一个球减去另一个球。
fn sd_death_star(p2: vec3f, ra: f32, rb: f32, d: f32) {
  // sampling independent computations (only depend on shape)
    let a = (ra * ra - rb * rb + d * d) / (2.0 * d);
    let b = sqrt(max(ra * ra - a * a, 0.0));
	
  // sampling dependant computations
    let p = vec2f(p2.x, length(p2.yz));

    return select(
        max((length(p) - ra),
            -(length(p - vec2(d, 0.0)) - rb)),
        length(p - vec2(a, b)),
        p.x * b - p.y * a > d * max(b - p.y, 0.0)
    );
}

// 平滑边缘的圆锥体
fn sd_vertical_round_cone(p: vec3f, r1: f32, r2: f32, h: f32) -> f32 {
  // sampling independent computations (only depend on shape)
    let b = (r1 - r2) / h;
    let a = sqrt(1.0 - b * b);

  // sampling dependant computations
    let q = vec2f(length(p.xz), p.y);
    let k = dot(q, vec2(-b, a));

    return select(
        select(
            dot(q, vec2(a, b)) - r1,
            length(q - vec2(0.0, h)) - r2,
            k > a * h
        ),
        length(q) - r1,
        k < 0.0
    );
}

// 边缘平滑的圆锥体
fn sd_round_cone(p: vec3f, a: vec3f, b: vec3f, r1: f32, r2: f32) -> f32 {
    // sampling independent computations (only depend on shape)
    let  ba = b - a;
    let l2 = dot(ba, ba);
    let rr = r1 - r2;
    let a2 = l2 - rr * rr;
    let il2 = 1.0 / l2;
    
    // sampling dependant computations
    let pa = p - a;
    let y = dot(pa, ba);
    let z = y - l2;
    let x2 = dot(pa * l2 - ba * y);
    let y2 = y * y * l2;
    let z2 = z * z * l2;

    // single square root!
    let k = sign(rr) * rr * rr * x2;

    return select(
        select(
            (sqrt(x2 * a2 * il2) + y * rr) * il2 - r1,
            sqrt(x2 + y2) * il2 - r1,
            sign(y) * a2 * y2 < k,
        ),
        sqrt(x2 + z2) * il2 - r2,
        sign(z) * a2 * z2 > k,
    );
}

// Ellipsoid - bound(not exact
// 椭球体
fn sd_ellipsoid(p: vec3f, r: vec3f) -> f32 {
    let k0 = length(p / r);
    let k1 = length(p / (r * r));
    return k0 * (k0 - 1.0) / k1;
}


// Revolved Vesica 
// 囊泡段
fn sd_vesica_segment(p: vec3f, a: vec3f, b: vec3f, w: f32) -> f32 {
    let c = (a + b) * 0.5;
    let l = length(b - a);
    let v = (b - a) / l;
    let y = dot(p - c, v);
    let q = vec2(length(p - c - y * v), abs(y));

    let r = 0.5 * l;
    let d = 0.5 * (r * r - w * w) / w;
    let h = select(
        vec3(-d, 0.0, d + w),
        vec3(0.0, r, 0.0),
        (r * q.x < d * (q.y - r)),
    );
    return length(q - h.xy) - h.z;
}

// 菱形
fn sd_rhombus(p: vec3f, la: f32, lb: f32, h: f32, ra: f32) -> f32 {
    let pp = abs(p);
    let b = vec2(la, lb);
    let f = clamp((ndot(b, b, -2.0 * pp.xz)) / dot(b, b), -1.0, 1.0);
    let q = vec2(length(pp.xz, -0.5 * b * vec2(1.0 - f, 1.0 + f)) * sign(pp.x * b.y + pp.z * b.x - b.x * b.y) - ra, pp.y - h);
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0));
}

// Octahedron
// 八面体
fn sd_octahedron(p: vec3f, s: f32) -> f32 {
    let pp = abs(p);
    let m = pp.x + pp.y + pp.z - s;
    let q = select(
        select(
            select(
                m * 0.57735027,
                pp.zxy,
                3.0 * pp.z < m,
            ),
            pp.yzx,
            3.0 * pp.y < m,
        ),
        pp.xyz,
        3.0 * pp.x < m,
    );

    if q == m * 0.57735027 {
        return q;
    }

    let k = clamp(0.5 * (q.z - q.y + s), 0.0, s);
    return length(vec3(q.x, q.y - s + k, q.z - k));
}


// Octahedron- bound (not exact)
// 八面体
fn sd_octahedron_not_exact(p: vec3f, s: f32) -> f32 {
    let pp = abs(p);
    return (pp.x + pp.y + pp.z - s) * 0.57735027;
}

// Pyramid - exact   (https://www.shadertoy.com/view/Ws3SDl)
// 金字塔
fn sd_pyramid(p: vec3f, h: f32) -> f32 {
    let m2 = h * h + 0.25;
    var pp = p;

    pp.x = abs(pp.x);
    pp.z = abs(pp.z);
    pp.x = select(pp.x, pp.z, pp.z > pp.x);
    pp.z = select(pp.z, pp.x, pp.z > pp.x);
    pp.x -= 0.5;
    pp.z -= 0.5;

    let q = vec3(pp.z, h * pp.y - 0.5 * pp.x, h * pp.x + 0.5 * pp.y);

    let s = max(-q.x, 0.0);
    let t = clamp((q.y - 0.5 * pp.z) / (m2 + 0.25), 0.0, 1.0);

    let a = m2 * (q.x + s) * (q.x + s) + q.y * q.y;
    let b = m2 * (q.x + 0.5 * t) * (q.x + 0.5 * t) + (q.y - m2 * t) * (q.y - m2 * t);

    let d2 = select(min(a, b), 0.0, min(q.y, -q.x * m2 - q.y * 0.5) > 0.0);

    return sqrt((d2 + q.z * q.z) / m2) * sign(max(q.z, -pp.y));
}


// Triangle
fn sd_triangle(p: vec3f, a: vec3f, b: vec3f, c: vec3f) -> f32 {
    let ba = b - a;
    let pa = p - a;
    let cb = c - b;
    let pb = p - b;
    let ac = a - c;
    let pc = p - c;
    let nor = cross(ba, ac);

    return sqrt(
        select(
            (sign(dot(cross(ba, nor), pa)) + sign(dot(cross(cb, nor), pb)) + sign(dot(cross(ac, nor), pc)) < 2.0),
            min(min(
                dot(ba * clamp(dot(ba, pa) / dot(ba), 0.0, 1.0) - pa),
                dot(cb * clamp(dot(cb, pb) / dot(cb), 0.0, 1.0) - pb)
            ),
                dot(ac * clamp(dot(ac, pc) / dot(ac), 0.0, 1.0) - pc)),
            dot(nor, pa) * dot(nor, pa) / dot(nor),
        )
    );
}


// 两个三角形组成的四边形
// Quad - exact(https,://www.shadertoy.com/view/Md2BWW)
fn sd_quad(p: vec3f, a: vec3f, b: vec3f, c: vec3f, d: vec3f) -> f32 {
    let ba = b - a;
    let pa = p - a;
    let cb = c - b;
    let pb = p - b;
    let dc = d - c;
    let pc = p - c;
    let ad = a - d;
    let pd = p - d;
    let nor = cross(ba, ad);

    return sqrt(
        select(
            dot(nor, pa) * dot(nor, pa) / dot(nor),
            min(
                min(
                    min(
                        dot(ba * clamp(dot(ba, pa) / dot(ba), 0.0, 1.0) - pa),
                        dot(cb * clamp(dot(cb, pb) / dot(cb), 0.0, 1.0) - pb)
                    ),
                    dot(dc * clamp(dot(dc, pc) / dot(dc), 0.0, 1.0) - pc)
                ),
                dot(ad * clamp(dot(ad, pd) / dot(ad), 0.0, 1.0) - pd)
            ),
            (sign(dot(cross(ba, nor), pa)) + sign(dot(cross(cb, nor), pb)) + sign(dot(cross(dc, nor), pc)) + sign(dot(cross(ad, nor), pd)) < 3.0)
        ),
    );
}