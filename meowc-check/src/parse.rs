use crate::check_sat::{Clause, Lit, RawLit, Var};
use std::{
	fs::File,
	io::{BufRead, BufReader},
	path::Path,
};

pub fn dimacs_iter<P: AsRef<Path>>(
	filename: P,
) -> (impl Iterator<Item = Clause>, Var) {
	let lines = BufReader::new(File::open(filename).unwrap()).lines();
	let mut lines = lines.skip_while(|l| l.as_ref().unwrap().starts_with("c"));
	let problem: Vec<String> = lines
		.next()
		.unwrap()
		.unwrap()
		.split_whitespace()
		.map(|s| s.to_string())
		.collect();

	// TODO: This is inelegant error handling
	assert!(problem.len() == 4);
	assert!(problem[0] == "p");
	assert!(problem[1] == "cnf");
	let variables = problem[2].parse::<u32>().unwrap();
	// TODO: Check number of clauses is right
	let clauses = problem[3].parse::<u32>().unwrap();

	// TODO: really we should iterate over numbers not lines, cos clauses can
	// take multiple lines
	let lines = lines.map(|line| {
		line.unwrap()
			.split_whitespace()
			.filter_map(|lit| {
				let lit = lit.parse::<RawLit>().unwrap();
				if lit == 0 {
					return None;
				}
				Some(Lit::from_dimacs_unchecked(lit))
			})
			.collect::<Clause>()
	});

	(lines, variables)
}

pub fn proof_iter<P: AsRef<Path>>(filename: P) -> impl Iterator<Item = Lit> {
	BufReader::new(File::open(filename).unwrap())
		.lines()
		.flat_map(|line| {
			line.unwrap()
				.split_whitespace()
				.map(|lit| Lit::from_dimacs_unchecked(lit.parse().unwrap()))
				// TODO: This seems nasty - can it be improved
				.collect::<Vec<_>>()
		})
}
