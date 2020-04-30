#[macro_use(c)]
extern crate cute;

mod colorize;
mod draw;
mod polygonize;
mod selection;

use gperftools::profiler::PROFILER;
use image;
use rand::distributions::uniform::SampleUniform;
use rand::prelude::*;
use selection::Mutation;
use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Activate profile mode
    #[structopt(short, long)]
    profile: bool,

    /// Input image
    #[structopt(name = "IMAGE", parse(from_os_str))]
    input_image: PathBuf,
}

pub fn randrange<T: SampleUniform>(l: T, h: T) -> T {
    rand::thread_rng().gen_range(l, h)
}

fn main() {
    let opt = Opt::from_args();
    let polygons_number = 100_000;
    let polygon_min = 6;
    let polygon_max = 10;
    let polygon_delta = 5;

    if opt.profile {
        PROFILER.lock().unwrap().start("./my-prof.prof").unwrap();
    }

    let ref_img = &image::open(opt.input_image).unwrap().to_rgb();

    let mut polygons = Mutation::<polygonize::Args, polygonize::DNA>::select(
        polygonize::Args {
            npolygons: polygons_number,
            pol_size: (polygon_min, polygon_max),
            pol_delta: polygon_delta,
            scale_muts: 100,
            width: 512,
            height: 512,
        },
        2,
    )
    .polygons;
    colorize::generate_colors(&mut polygons, ref_img);

    let mut scale_muts = (polygons_number / 2) as usize;

    while scale_muts > 4 {
        let divide_rate = (scale_muts * 10) as u64;
        eprintln!("\nmuts {}\n", scale_muts);

        polygons = Mutation::<colorize::Args, colorize::DNA>::select(
            colorize::Args {
                ref_img,
                polygons,
                scale_muts,
                divide_rate,
            },
            2,
        )
        .polygons;

        scale_muts /= 2;
    }

    let black = draw::Color(image::Rgb([0, 0, 0]));
    let image = colorize::polygons_draw(512, 512, black, polygons);

    image
        .save_with_format("output.png", image::ImageFormat::Png)
        .unwrap();

    if opt.profile {
        PROFILER.lock().unwrap().stop().unwrap();
    }
}
