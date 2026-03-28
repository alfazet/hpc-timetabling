use crate::{
    model::TimetableData,
    solver::{NaiveSolver, Solver},
};
use anyhow::{Result, bail};
use parser::problem::Problem;
use rand::rng;
use serializer::output::OutputMetadata;
use std::{env, fs};

mod model;
mod output;
mod solution;
mod solver;
mod fitness;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        bail!("usage: {} <problem_data.xml>", args[0]);
    }

    let input = fs::read_to_string(&args[1])?;
    let problem = Problem::parse(input)?;
    let output_metadata = OutputMetadata::from_problem(&problem);

    let data = TimetableData::new(problem);
    let population_size = 16000;
    let generations = 40;
    let mut solver = NaiveSolver::new(Box::new(rng()), population_size, generations, data.clone());
    let solution = solver.solve();

    let output = output::output(&solution.inner, &data);
    // dbg!(&output);
    let Some(output) = output else {
        eprintln!("No valid solution found!");
        return Ok(());
    };

    let xml_solution = output.serialize(output_metadata);

    // keep only xml in stdout, debug info in stderr to allow for uses like
    // `cargo r problem.xml > solution.xml`
    println!("{}", xml_solution);
    eprintln!("best fitness: {}", solution.fitness);

    Ok(())
}
