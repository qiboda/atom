use std::ops::RangeInclusive;

use bevy::{
    log::warn,
    math::{UVec2, Vec2},
};

// return p is in p0, p1, p2 triangle
pub fn point_in_triangle(p: Vec2, p0: Vec2, p1: Vec2, p2: Vec2) -> bool {
    let s = (p0.x - p2.x) * (p.y - p2.y) - (p0.y - p2.y) * (p.x - p2.x);
    let t = (p1.x - p0.x) * (p.y - p0.y) - (p1.y - p0.y) * (p.x - p0.x);

    if (s < 0.0) != (t < 0.0) && s != 0.0 && t != 0.0 {
        return false;
    }

    let d = (p2.x - p1.x) * (p.y - p1.y) - (p2.y - p1.y) * (p.x - p1.x);
    d == 0.0 || (d < 0.0) == (s + t <= 0.0)
}

// p is in p0, p1, p2 triangle, return the interpolation value
pub fn triangle_interpolation(
    p: Vec2,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    v0: f32,
    v1: f32,
    v2: f32,
) -> f32 {
    let area = (p1.y - p2.y) * (p0.x - p2.x) + (p2.x - p1.x) * (p0.y - p2.y);
    let area_inv = 1.0 / area;

    let s = ((p1.y - p2.y) * (p.x - p2.x) + (p2.x - p1.x) * (p.y - p2.y)) * area_inv;
    let t = ((p2.y - p0.y) * (p.x - p2.x) + (p0.x - p2.x) * (p.y - p2.y)) * area_inv;
    v0 * s + v1 * t + v2 * (1.0 - s - t)
}

pub fn points_in_triangle(mut pt0: Vec2, mut pt1: Vec2, mut pt2: Vec2) -> Vec<UVec2> {
    let mut points = vec![];
    /*
        // https://www.geeksforgeeks.org/check-whether-triangle-valid-not-sides-given/
        a + b > c
        a + c > b
        b + c > a
    */

    let a = pt0.distance(pt1);
    let b = pt1.distance(pt2);
    let c = pt2.distance(pt0);

    if a + b <= c || a + c <= b || b + c <= a {
        warn!(
            "The given points must form a triangle. [{}, {}, {}]",
            pt0, pt1, pt2
        );
        return points;
    }

    if triangle_area(pt0, pt1, pt2) <= 1.0 {
        let center = get_triangle_center(pt0, pt1, pt2);
        points.push(center.as_uvec2());
        return points;
    }

    // p1 p2 p3 从右到左排序
    if pt1.x < pt0.x {
        (pt0, pt1) = (pt1, pt0);
    }

    if pt2.x < pt1.x {
        (pt1, pt2) = (pt2, pt1);

        if pt1.x < pt0.x {
            (pt1, pt0) = (pt0, pt1);
        }
    }

    pt0 = pt0.floor();
    pt1 = pt1.floor();
    pt2 = pt2.floor();

    // 线段1 最长的
    let base_func = line_y_func(pt0, pt2);

    // 线段2
    let line1_func = line_y_func(pt0, pt1);

    for x in pt0.x as u32..pt1.x as u32 {
        let y_range = get_range(line1_func(x as f32), base_func(x as f32));

        for y in y_range {
            points.push(UVec2::new(x, y));
        }
    }

    // 线段3
    let line2_func = line_y_func_2(pt1, pt2);
    for x in pt1.x as u32..=pt2.x as u32 {
        let y_range: RangeInclusive<u32> = get_range(line2_func(x as f32), base_func(x as f32));

        for y in y_range {
            points.push(UVec2::new(x, y));
        }
    }

    points
}

// pt1 to pt2 line: get y from x
fn line_y_func(pt0: Vec2, pt1: Vec2) -> impl Fn(f32) -> f32 {
    move |x| {
        if pt1.x.round() == pt0.x.round() {
            return pt1.y;
        }
        let m = (pt1.y - pt0.y) / (pt1.x - pt0.x);
        m * (x - pt0.x) + pt0.y
    }
}

fn line_y_func_2(pt0: Vec2, pt1: Vec2) -> impl Fn(f32) -> f32 {
    move |x| {
        if pt1.x.round() == pt0.x.round() {
            return pt0.y;
        }
        let m = (pt1.y - pt0.y) / (pt1.x - pt0.x);
        m * (x - pt0.x) + pt0.y
    }
}

fn get_range(y0: f32, y1: f32) -> RangeInclusive<u32> {
    if y0 < y1 {
        y0.ceil() as u32..=y1.floor() as u32
    } else {
        y1.ceil() as u32..=y0.floor() as u32
    }
}

pub fn triangle_area(p0: Vec2, p1: Vec2, p2: Vec2) -> f32 {
    let mut a = 0.0;
    let mut b = 0.0;
    let mut c = 0.0;

    if !check_if_valid_triangle(p0, p1, p2, &mut a, &mut b, &mut c) {
        warn!(
            "The given points is not a valid triangle. [{}, {}, {}]",
            p0, p1, p2
        );
        return 0.0;
    }

    triangle_area_edge(a, b, c)
}

pub fn triangle_area_edge(a: f32, b: f32, c: f32) -> f32 {
    // Thanks to: http://james-ramsden.com/area-of-a-triangle-in-3d-c-code/

    let s = (a + b + c) / 2.0;
    (s * (s - a) * (s - b) * (s - c)).sqrt()
}

pub fn get_triangle_center(p0: Vec2, p1: Vec2, p2: Vec2) -> Vec2 {
    // Thanks to: https://stackoverflow.com/questions/524755/finding-center-of-2d-triangle
    (p0 + p1 + p2) / 3.0
}

pub fn check_if_valid_triangle(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    a: &mut f32,
    b: &mut f32,
    c: &mut f32,
) -> bool {
    *a = p0.distance(p1);
    *b = p1.distance(p2);
    *c = p2.distance(p0);

    !(*a + *b <= *c || *a + *c <= *b || *b + *c <= *a)
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use crate::math::points_in_triangle;

    #[test]
    fn test_range() {
        let p0 = Vec2::new(513.679, 645.605);
        let p1 = Vec2::new(513.683, 636.393);
        let p2 = Vec2::new(507.004, 641.961);
        let points = points_in_triangle(p0, p1, p2);
        println!("{:?}", points);
    }

    #[test]
    fn test_triangle_interpolation() {
        // p2(0)
        // p0(0)  p1(1)
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(1.0, 0.0);
        let p2 = Vec2::new(0.0, 1.0);

        let v0 = 0.0;
        let v1 = 1.0;
        let v2 = 0.0;

        let p = Vec2::new(0.5, 0.5);
        let result = super::triangle_interpolation(p, p0, p1, p2, v0, v1, v2);
        assert_eq!(result, 0.5);

        let p = Vec2::new(0.0, 0.0);
        let result = super::triangle_interpolation(p, p0, p1, p2, v0, v1, v2);
        assert_eq!(result, 0.0);
    }
}
