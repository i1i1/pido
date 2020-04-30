use super::{draw::*, selection::*, *};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Colors {
    // 512 of those
    cols: HashMap<Color, i32>,
    n: i32,
}

#[derive(Debug, Clone)]
pub struct DNA<'a> {
    width: u32,
    height: u32,
    ref_img: &'a image::RgbImage,
    colors: Colors,
    bg: Color,
    scale_muts: usize,
    pub polygons: Vec<Polygon>,
    divide_rate: u64,
}

#[derive(Clone)]
pub struct Args<'a> {
    pub ref_img: &'a image::RgbImage,
    pub scale_muts: usize,
    pub polygons: Vec<Polygon>,
    pub divide_rate: u64,
}

impl Colors {
    fn get_bg(&self) -> Color {
        let mut max = None;
        let mut color = None;

        for (col, n) in &self.cols {
            if max == None || max.unwrap() < n {
                max = Some(n);
                color = Some(col);
            }
        }

        *color.unwrap()
    }

    fn generate(&self) -> Color {
        let n = randrange(0, self.cols.len());
        *self.cols.iter().nth(n).unwrap().0
    }

    fn new(img: &image::RgbImage) -> Self {
        let mut colors = Colors {
            cols: Default::default(),
            n: 0,
        };

        for p in img.pixels() {
            colors
                .cols
                .entry(Color(*p))
                .and_modify(|e| *e += 1)
                .or_insert(1);
            colors.n += 1;
        }

        colors
    }
}

///
/// Paint all DNA polygons onto an Image and show it.
///
pub fn polygons_draw(
    width: u32,
    height: u32,
    bg: Color,
    polygons: Vec<Polygon>,
) -> image::RgbImage {
    fn get_average_color(p: Vec<image::Rgb<u8>>) -> image::Rgb<u8> {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;

        for c in &p {
            r += c[0] as u64;
            g += c[1] as u64;
            b += c[2] as u64;
        }

        image::Rgb([
            (r / p.len() as u64) as u8,
            (g / p.len() as u64) as u8,
            (b / p.len() as u64) as u8,
        ])
    }

    let mut img = image::ImageBuffer::from_pixel(width, height, bg.0);
    let mut pixels = vec![Vec::new(); (width * height) as usize];

    for p in &polygons {
        //p.draw(&mut img);
        let color = p.color.0;
        for (x, y) in p.iter_points(width as i32, height as i32) {
            pixels[(x * (width as i32) + y) as usize].push(color);
        }
    }

    for x in 0..width {
        for y in 0..height {
            let p = pixels[(x * width + y) as usize].clone();
            let c = if p.len() == 0 {
                bg.0
            } else {
                get_average_color(p)
            };

            img.put_pixel(x as u32, y, c);
        }
    }

    img
}

impl<'a> Selection<Args<'a>> for DNA<'a> {
    ///
    /// Generate dna string consisting of polygons.
    ///
    fn generate(args: Args<'a>) -> Self {
        let (width, height) = args.ref_img.dimensions();
        let colors = Colors::new(&args.ref_img);
        let bg = colors.get_bg();

        DNA {
            colors,
            bg,
            width,
            height,
            ref_img: args.ref_img,
            scale_muts: args.scale_muts,
            polygons: args.polygons,
            divide_rate: args.divide_rate,
        }
    }

    fn mutate(&self) -> Self {
        let mut dna = self.clone();
        let mut scores: Vec<(usize, u64)> = dna
            .polygons
            .iter()
            .enumerate()
            .map(|(n, p)| (n, p.score(self.ref_img)))
            .collect();

        scores.sort_by_key(|args: &(usize, u64)| args.1);
        let indexes =
            scores.iter().map(|(n, _)| *n).rev().collect::<Vec<usize>>();
        let indexes = &indexes[..self.scale_muts];

        for i in indexes {
            dna.polygons[*i].color = self.colors.generate();
        }

        dna
    }

    fn fitness(&self) -> u64 {
        let img1 = polygons_draw(
            self.width,
            self.height,
            self.bg,
            self.polygons.clone(),
        );
        let img2 = self.ref_img;
        let (width, height) = img1.dimensions();
        let mut fitness = 0;

        for y in 0..height {
            for x in 0..width {
                let image::Rgb([r1, g1, b1]) = *img1.get_pixel(x, y);
                let image::Rgb([r2, g2, b2]) = *img2.get_pixel(x, y);
                let d_r = (r1 as i32 - r2 as i32) as f64;
                let d_g = (g1 as i32 - g2 as i32) as f64;
                let d_b = (b1 as i32 - b2 as i32) as f64;

                fitness += (d_r * d_r + d_g * d_g + d_b * d_b).sqrt() as u64;
            }
        }

        fitness / self.divide_rate
    }

    fn print(&self, ngen: u64, fitness: u64) {
        let print = (fitness * self.divide_rate) as f64
            / self.width as f64
            / self.height as f64;
        let print = print * print;
        let print = (print / 3.).sqrt();
        println!(
            "colorize_first: Showing generation {} fitness: {} {}",
            ngen, print, fitness
        );

        let img = polygons_draw(
            self.width,
            self.height,
            self.bg,
            self.polygons.clone(),
        );

        img.save_with_format(
            format!("./img/{:0>4}.png", ngen),
            image::ImageFormat::Png,
        )
        .unwrap();
    }
}

pub fn generate_colors(pol: &mut Vec<Polygon>, ref_img: &image::RgbImage) {
    let colors = Colors::new(&ref_img);

    for mut p in pol {
        p.color = colors.generate();
    }
}
