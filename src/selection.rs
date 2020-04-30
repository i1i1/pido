use rayon::iter::*;
use rayon::slice::ParallelSliceMut;

pub trait Selection<A: Send + Sync + Clone>:
    Send + Sync + Clone + Sized
{
    fn mutate(&self) -> Self;
    fn generate(args: A) -> Self;
    fn fitness(&self) -> u64;
    fn print(&self, ngen: u64, fitness: u64);
}

#[derive(Debug, Clone)]
pub struct Mutation<Args: Send + Sync + Clone, DNA: Selection<Args>> {
    dna: DNA,
    fitness: u64,
    marker: std::marker::PhantomData<Args>,
}

impl<Args: Send + Sync + Clone, DNA: Selection<Args>> Mutation<Args, DNA> {
    fn new_gen(&self, nmuts: usize) -> Vec<Mutation<Args, DNA>> {
        let mutations = c![self.dna.mutate(), for _i in 0..nmuts];

        let mut mutations = mutations
            .par_iter()
            .map(|dna| Self {
                dna: dna.clone(),
                fitness: dna.fitness(),
                marker: std::marker::PhantomData,
            })
            .collect::<Vec<Self>>();

        mutations.par_sort_by_key(|it| it.fitness);
        mutations[..nmuts].to_vec()
    }

    pub fn select(args: Args, retries: i32) -> DNA {
        let parent = {
            let dna = DNA::generate(args);
            let fitness = dna.fitness();

            Mutation {
                dna,
                fitness,
                marker: std::marker::PhantomData,
            }
        };
        let mut nmuts = 10;
        let mut fails = 0;
        let mut gen = parent.new_gen(nmuts);
        let mut last_fitness = 1_000_000_000;
        let mut ngen = 0;

        loop {
            ngen += 1;

            let mut kids = gen.par_iter().map(|p| p.new_gen(nmuts)).reduce(
                || Vec::new(),
                |mut a: Vec<Self>, mut b: Vec<Self>| {
                    a.append(&mut b);
                    a
                },
            );
            kids.append(&mut gen);
            kids.par_sort_by_key(|it| it.fitness);
            gen = kids[..nmuts].to_vec();

            let min_kid = &gen[0];
            if min_kid.fitness == last_fitness {
                nmuts += 1;
                fails += 1;
                println!("nmuts {}", nmuts);
            } else {
                fails = 0;
            }
            if fails == retries {
                break;
            }
            min_kid.dna.print(ngen, min_kid.fitness);
            last_fitness = min_kid.fitness;
        }

        let min_kid = &gen[0];
        min_kid.dna.clone()
    }
}
