use anyhow::Result;
use clap::{arg, Parser, ValueHint::FilePath};
use meowc_check_core::check_sat::{check_sat, SatResult};
use std::path::PathBuf;

mod parse;
use parse::{dimacs_iter, proof_iter};

#[derive(Parser, Debug)]
struct Args {
	#[arg(value_hint = FilePath)]
	dimacs_file: PathBuf,
	#[arg(value_hint = FilePath)]
	proof_file: PathBuf,
}

fn main() -> Result<()> {
	let args = Args::parse();

	println!("c Checking SAT proof");
	let (dimacs, max_var) = dimacs_iter(args.dimacs_file)?;
	let proof = proof_iter(args.proof_file)?;
	let result = check_sat(dimacs, proof, max_var);

	if let SatResult::Verified = result {
		println!("s VERIFIED");
	} else {
		println!("s NOT VERIFIED");
	};

	Ok(())
}
