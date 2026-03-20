use anyhow::{Result, bail};
use std::{env, fs};
use rand::rng;
use parser::problem::Problem;
use crate::{model::TimetableData, output::Output, solver::{NaiveSolver, Solver}};

mod model;
mod assigner;
mod solver;
mod output;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        bail!("usage: {} <problem_data>", args[0]);
    }

    let input = fs::read_to_string(&args[1])?;
    let problem = Problem::parse(input)?;
    // println!("{:#?}", problem);
    
    let data = TimetableData::new(&problem);
    let mut solver = NaiveSolver::new(Box::new(rng()), 1, 1, data.clone());
    let solution = solver.solve();
    dbg!(&solution);

    let output = Output::new(solution, data);
    dbg!(output);

    Ok(())
}
