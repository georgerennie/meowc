use meowc_check_core::check_sat::{check_sat, SatResult};
use std::env;

mod parse;
use parse::{dimacs_iter, proof_iter};

fn main() {
	let args: Vec<_> = env::args().collect();
	assert!(args.len() == 3);
	println!("c Checking SAT proof");
	let (dimacs, max_var) = dimacs_iter(&args[1]);
	let proof = proof_iter(&args[2]);
	let result = check_sat(dimacs, proof, max_var);

	if let SatResult::Verified = result {
		println!("s VERIFIED");
	} else {
		println!("s NOT VERIFIED");
	}
}
