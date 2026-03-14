use anyhow::{Result, bail};
use std::{env, fs};

use parser::problem::Problem;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        bail!("usage: {} <problem_data>", args[0]);
    }

    let input = fs::read_to_string(&args[1])?;
    let problem = Problem::parse(input)?;
    println!("{:#?}", problem);

    Ok(())
}
