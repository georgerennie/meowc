use meowc_check::check_sat::{check_sat, SatResult};
use std::env;

#[cfg(not(feature = "contracts"))]
use meowc_check::parse::{dimacs_iter, proof_iter};

// TODO: Work out how to cleanly feature gate this stuff - maybe we should
// split into two crates, one for the proven core and one for everything else

#[cfg(not(feature = "contracts"))]
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

#[cfg(feature = "contracts")]
fn main() {}
