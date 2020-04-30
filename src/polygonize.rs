use super::{draw::*, selection::*};

#[derive(Debug, Clone)]
pub struct DNA {
    width: i32,
    height: i32,
    pub polygons: Vec<Polygon>,
    scale_muts: usize,
    pol_size: (i32, i32),
    pol_delta: i32,
    pixels: Vec<u8>,
    fitness: u64,
}

#[derive(Clone)]
pub struct Args {
    pub npolygons: i32,
    pub pol_size: (i32, i32),
    pub pol_delta: i32,
    pub scale_muts: usize,
    pub width: i32,
    pub height: i32,
}

impl DNA {
    fn calculate_real_fitness(&mut self) {
        self.fitness = 0;

        for p in &self.pixels {
            let p = *p as u64;
            self.fitness += match p {
                0 => 1,
                _ => 0,
            }
        }
    }

    fn calculate_fitness(&mut self) {
        for p in &self.polygons {
            for (x, y) in p.iter_points(self.width, self.height) {
                self.pixels[(y * self.width + x) as usize] += 1;
            }
        }

        self.calculate_real_fitness();
    }

    fn calculate_delta_fitness(
        &mut self,
        mut old_polygons: Vec<Polygon>,
        mut new_polygons: Vec<Polygon>,
    ) {
        while let Some(p) = old_polygons.pop() {
            for (x, y) in p.iter_points(self.width, self.height) {
                self.pixels[(y * self.width + x) as usize] -= 1;
            }
        }

        while let Some(p) = new_polygons.pop() {
            for (x, y) in p.iter_points(self.width, self.height) {
                self.pixels[(y * self.width + x) as usize] += 1;
            }
            self.polygons.push(p);
        }

        self.calculate_real_fitness();
    }

    fn score_polygon(&self, polygon: &Polygon) -> u64 {
        let mut score = 0;

        for (x, y) in polygon.iter_points(self.width as i32, self.height as i32)
        {
            score += self.pixels[(y * self.width + x) as usize] as u64;
        }
        score
    }
}

impl Selection<Args> for DNA {
    fn generate(args: Args) -> Self {
        let polygons = c![
            Polygon::generate(
                args.width,
                args.height,
                args.pol_size,
                args.pol_delta
            ),
            for _i in 0..args.npolygons
        ];
        let mut g = Self {
            polygons,
            width: args.width,
            height: args.height,
            pol_size: args.pol_size,
            pol_delta: args.pol_delta,
            scale_muts: args.scale_muts,
            pixels: vec![0u8; args.width as usize * args.height as usize],
            fitness: 0,
        };
        g.calculate_fitness();
        g
    }

    fn mutate(&self) -> Self {
        let mut dna = self.clone();
        dna.polygons.sort_by_key(|p| self.score_polygon(&p));

        let mut old_polygons = Vec::new();
        let new_polygons = c![
            Polygon::generate(
                self.width,
                self.height,
                self.pol_size,
                self.pol_delta
            ),
            for _i in 0..self.scale_muts
        ];

        for _ in 0..self.scale_muts {
            old_polygons.push(dna.polygons.pop().unwrap());
        }

        dna.calculate_delta_fitness(old_polygons, new_polygons);

        dna
    }

    fn fitness(&self) -> u64 {
        self.fitness
    }

    fn print(&self, ngen: u64, fitness: u64) {
        let mut exact = 0;
        let mut empty = 0;
        let mut rest = 0;

        for p in &self.pixels {
            match *p as u64 {
                1 => exact += 1,
                0 => empty += 1,
                _ => rest += 1,
            }
        }
        println!(
            "polygonize: Showing generation {} fitness: {} exact {} empty {} rest {}",
            ngen, fitness, exact, empty, rest
        );
    }
}
