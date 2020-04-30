use super::randrange;
use derive_more::{Add, Sub};
use image;
use std::cmp::*;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct Color(pub image::Rgb<u8>);

#[derive(Copy, Clone, Add, Sub, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct Polygon {
    pub color: Color,
    pub points: [Point; 3],
}

// impl Color {
//     pub fn near(&mut self, range: i16) {
//         let Color(image::Rgb(arr)) = self;
//
//         arr[0] = (arr[0] as i16 + randrange(-range, range)) as u8;
//         arr[1] = (arr[1] as i16 + randrange(-range, range)) as u8;
//         arr[2] = (arr[2] as i16 + randrange(-range, range)) as u8;
//     }
// }

impl Point {
    const OFFSET: i32 = 3;

    fn generate(width: i32, height: i32) -> Self {
        Point {
            x: randrange(0 - Point::OFFSET, width + Point::OFFSET),
            y: randrange(0 - Point::OFFSET, height + Point::OFFSET),
        }
    }

    fn generate_near(self, width: i32, height: i32) -> Self {
        let (width, height) = (width.abs(), height.abs());
        Point {
            x: self.x + randrange(-width, width),
            y: self.y + randrange(-height, height),
        }
    }
}

pub struct PolygonIterator {
    width: i32,
    height: i32,
    p0: Point,
    p1: Point,
    p2: Point,

    i: i32,
    j: i32,
    inter: Option<(i32, i32)>,
}

impl Iterator for PolygonIterator {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        let total_height = self.p2.y - self.p0.y;

        loop {
            if self.i == total_height {
                return None;
            }

            if self.inter == None {
                let second_half =
                    self.i > self.p1.y - self.p0.y || self.p1.y == self.p0.y;
                let segment_height = if second_half {
                    self.p2.y - self.p1.y
                } else {
                    self.p1.y - self.p0.y
                };
                let alpha: f64 = (self.i as f64) / (total_height as f64);
                let tmp = if second_half {
                    self.p1.y - self.p0.y
                } else {
                    0
                };
                // be careful: with above conditions no division by zero here
                let beta: f64 = (self.i - tmp) as f64 / segment_height as f64;

                let a_mat = ((self.p0.x as f64)
                    + ((self.p2.x - self.p0.x) as f64) * alpha)
                    as i32;

                let b_mat = if second_half {
                    (self.p1.x as f64 + (self.p2.x - self.p1.x) as f64 * beta)
                        as i32
                } else {
                    (self.p0.x as f64 + (self.p1.x - self.p0.x) as f64 * beta)
                        as i32
                };

                let (a_mat, b_mat) = if a_mat > b_mat {
                    (b_mat, a_mat)
                } else {
                    (a_mat, b_mat)
                };

                self.inter = Some((a_mat, b_mat));
            }

            let (a_mat, b_mat) = self.inter.unwrap();
            let y = self.p0.y + self.i;

            loop {
                if self.j == b_mat - a_mat {
                    self.j = 0;
                    break;
                }

                let x = a_mat + self.j;
                self.j += 1;
                if x >= 0 && x < self.width && y >= 0 && y < self.height {
                    return Some((x, y));
                }
            }

            self.inter = None;
            self.i += 1;
        }
    }
}

impl Polygon {
    const COLOR_BLACK: Color = Color(image::Rgb([255, 255, 255]));

    ///
    /// Drawing algorithm from here: https://habr.com/en/post/248159/
    ///
    pub fn iter_points(&self, width: i32, height: i32) -> PolygonIterator {
        let (p0, p1, p2) = (self.points[0], self.points[1], self.points[2]);

        PolygonIterator {
            width,
            height,
            p0,
            p1,
            p2,

            i: 0,
            j: 0,
            inter: None,
        }
    }

    pub fn score(&self, ref_img: &image::RgbImage) -> u64 {
        let (width, height) = ref_img.dimensions();
        let Color(image::Rgb([r, g, b])) = self.color;
        let mut score = 0;
        for (x, y) in self.iter_points(width as i32, height as i32) {
            let image::Rgb([r1, g1, b1]) =
                *ref_img.get_pixel(x as u32, y as u32);
            let d_r = (r as i32 - r1 as i32) as f64;
            let d_g = (g as i32 - g1 as i32) as f64;
            let d_b = (b as i32 - b1 as i32) as f64;

            score += (d_r * d_r + d_g * d_g + d_b * d_b).sqrt() as u64;
        }
        score
    }

    pub fn generate(
        img_w: i32,
        img_h: i32,
        size: (i32, i32),
        delta: i32,
    ) -> Self {
        fn generate_points(
            img_w: i32,
            img_h: i32,
            max_size: i32,
        ) -> [Point; 3] {
            let p = Point::generate(img_w, img_h);
            [
                p,
                p.generate_near(-max_size, max_size),
                p.generate_near(-max_size, max_size),
            ]
        }
        fn is_degenerate(points: &[Point]) -> bool {
            points[0].y == points[1].y && points[0].y == points[2].y
        }
        fn is_fit(points: &[Point], size: (i32, i32), delta: i32) -> bool {
            let min_x = points.iter().map(|p| p.x).min().unwrap();
            let max_x = points.iter().map(|p| p.x).max().unwrap();
            let min_y = points.iter().map(|p| p.y).min().unwrap();
            let max_y = points.iter().map(|p| p.y).max().unwrap();

            let (dx, dy) = (max_x - min_x, max_y - min_y);

            (dx - dy).abs() < delta && dx > size.0 && dy > size.0
        }

        let mut points = loop {
            let points = generate_points(img_w, img_h, size.1);
            if !is_degenerate(&points) && is_fit(&points, size, delta) {
                break points;
            }
        };

        points.sort_by_key(|p| p.y);

        Polygon {
            color: Polygon::COLOR_BLACK,
            points,
        }
    }
}
