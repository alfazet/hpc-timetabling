use crate::{
    crossover::OnePointCrossover,
    elitism::Elitism,
    model::TimetableData,
    mutation::BasicMutation,
    selection::TournamentSelection,
    solver::{NaiveSolver, Solver},
};
use anyhow::{Context, Result};
use clap::Parser;
use parser::problem::Problem;
use serializer::output::OutputMetadata;
use std::fs;

mod assigner;
mod crossover;
mod distribution;
mod elitism;
mod model;
mod mutation;
mod output;
mod penalty;
mod selection;
mod solution;
mod solver;
mod adjuster;
mod utils;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "ITC2019 timetable solver",
    long_about = "Solves ITC2019 timetabling problems using a genetic algorithm.\n\
                  \n\
                  You can provide parameters either positionally:\n\
                  solver <input> [generations] [population_size] [mutation_rate] [elitism]\n\
                  \n\
                  Or via flags:\n\
                  solver <input> --generations 100 --population-size 1000"
)]
struct Args {
    /// problem file in itc2019 XML format
    input: String,

    /// number of generations of the algorithm
    #[arg(short, long, default_value_t = 200)]
    generations: usize,

    /// initial population size
    #[arg(short, long, default_value_t = 8000)]
    population_size: usize,

    /// probability of a crossover between two parents occuring
    #[arg(short, long, default_value_t = 0.9)]
    crossover_rate: f32,

    /// probability of a mutation occuring
    #[arg(short, long, default_value_t = 0.05)]
    mutation_rate: f32,

    /// fraction of best solutions to keep unchanged every generation
    #[arg(short, long, default_value_t = 0.01)]
    elitism: f32,

    /// fraction of the population to use for tournament size
    #[arg(short, long, default_value_t = 0.02)]
    tournament_frac: f32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", &args.input))?;

    let problem = Problem::parse(input)?;
    let output_metadata = OutputMetadata::from_problem(&problem);
    let data = TimetableData::new(problem);

    let mut solver = NaiveSolver::new(
        args.population_size,
        args.generations,
        data.clone(),
        Elitism::new(args.elitism),
        TournamentSelection::new(((args.population_size as f32 * args.tournament_frac) as usize).max(1)),
        OnePointCrossover::new(args.crossover_rate),
        BasicMutation::new(args.mutation_rate),
    );

    eprintln!(
        "Solving '{}' with {} generations of {} individuals",
        args.input, args.generations, args.population_size
    );
    let mut rng = rand::rng();
    let solution = solver.solve(&mut rng);

    let output = output::output(&solution.inner, &solution.student_assignment, &data);
    let Some(output) = output else {
        eprintln!("no valid solution found!");
        return Ok(());
    };

    let xml_solution = output.serialize(output_metadata);
    // keep only xml in stdout, debug info in stderr to allow for uses like
    // `cargo r problem.xml > solution.xml`
    println!("{}", xml_solution);
    eprintln!("best fitness: {}", solution.penalty);

    Ok(())
}
