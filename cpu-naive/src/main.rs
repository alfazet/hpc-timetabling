use crate::{
    crossover::OnePointCrossover,
    elitism::Elitism,
    model::TimetableData,
    mutation::BasicMutation,
    selection::TournamentSelection,
    solver::{NaiveSolver, Solver},
};
use anyhow::{Context, Result, bail};
use parser::problem::Problem;
use rand::{SeedableRng, rngs::StdRng};
use serializer::output::OutputMetadata;
use std::{cell::RefCell, env, fs, rc::Rc};

mod assigner;
mod crossover;
mod distribution;
mod elitism;
mod fitness;
mod model;
mod mutation;
mod output;
mod selection;
mod solution;
mod solver;

fn main() -> Result<()> {
    let mut generations = 200;
    let mut population_size = 8000;
    let mut mutation_rate = 0.05;
    let mut elitism = 0.01;

    // TODO: refactor with clap
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        bail!(
            "usage: {} <problem_data.xml> [generations | {}] [population_size | {}] [mutation_rate | {}] [elitism | {}]",
            args[0],
            generations,
            population_size,
            mutation_rate,
            elitism,
        );
    }

    let input = fs::read_to_string(&args[1])?;
    let problem = Problem::parse(input)?;
    let output_metadata = OutputMetadata::from_problem(&problem);
    let data = TimetableData::new(problem);

    if args.len() >= 3 {
        generations = args[2]
            .parse::<usize>()
            .with_context(|| format!("invalid generations value '{}'", &args[2]))?;
    }
    if args.len() >= 4 {
        population_size = args[3]
            .parse::<usize>()
            .with_context(|| format!("invalid population_size value '{}'", &args[3]))?;
    }
    if args.len() >= 5 {
        mutation_rate = args[4]
            .parse::<f32>()
            .with_context(|| format!("invalid mutation_rate value '{}'", &args[4]))?;
    }
    if args.len() >= 6 {
        elitism = args[5]
            .parse::<f32>()
            .with_context(|| format!("invalid elitism value '{}'", &args[5]))?;
    }

    let mut solver = NaiveSolver::new(
        population_size,
        generations,
        data.clone(),
        Elitism::new(elitism),
        TournamentSelection::new((population_size / 100).max(1)),
        OnePointCrossover::new(),
        BasicMutation::new(mutation_rate),
    );

    eprintln!(
        "Solving '{}' with {} generations of {} individuals",
        args[1], generations, population_size
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
    eprintln!("best fitness: {}", solution.fitness);

    Ok(())
}
