use std::io::prelude::*;
use std::{env, fs::File};

fn main() {
	let args: Vec<_> = env::args().collect();
	let mut file = File::open(&args[1]).unwrap();
	let mut contents = String::new();
	file.read_to_string(&mut contents).unwrap();

	let mut solver = meowc_sat::dimacs_cnf::parse_dimacs(&contents).unwrap();

	println!("{:?}", solver.solve());
}
