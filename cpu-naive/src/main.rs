use crate::{
    crossover::UniformCrossover,
    elitism::Elitism,
    model::TimetableData,
    mutation::BasicMutation,
    selection::TournamentSelection,
    solver::{NaiveSolver, Solver},
};
use anyhow::{Result, bail};
use parser::problem::Problem;
use serializer::output::OutputMetadata;
use std::{env, fs};

mod crossover;
mod elitism;
mod fitness;
mod model;
mod mutation;
mod output;
mod selection;
mod solution;
mod solver;
mod distribution;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        bail!("usage: {} <problem_data.xml>", args[0]);
    }

    let input = fs::read_to_string(&args[1])?;
    let problem = Problem::parse(input)?;
    let output_metadata = OutputMetadata::from_problem(&problem);

    let data = TimetableData::new(problem);
    let population_size = 8000;
    let generations = 30;
    let rng = Box::new(rand::rng());
    let mut solver = NaiveSolver::new(
        rng.clone(),
        population_size,
        generations,
        data.clone(),
        Elitism::new(0.01),
        TournamentSelection::new(rng.clone(), 5),
        UniformCrossover::new(rng.clone()),
        BasicMutation::new(rng, 0.03),
    );
    let solution = solver.solve();

    let output = output::output(&solution.inner, &data);
    let Some(output) = output else {
        eprintln!("no valid solution found!");
        return Ok(());
    };

    let xml_solution = output.serialize(output_metadata);

    // keep only xml in stdout, debug info in stderr to allow for uses like
    // `cargo r problem.xml > solution.xml`
    println!("{}", xml_solution);
    eprintln!("best fitness: {}", solution.fitness);

    Ok(())
}
