use anyhow::Result;
use meowc_check_core::check_sat::{Clause, Lit, RawLit, Var};
use std::{
	fs::File,
	io::{BufRead, BufReader},
	path::Path,
};

pub fn dimacs_iter<P: AsRef<Path>>(
	filename: P,
) -> Result<(impl Iterator<Item = Clause>, Var, usize)> {
	let lines = BufReader::new(File::open(filename)?).lines();
	let mut lines = lines.skip_while(|l| l.as_ref().unwrap().starts_with("c"));
	let problem: Vec<String> = lines
		.next()
		.unwrap()?
		.split_whitespace()
		.map(|s| s.to_string())
		.collect();

	// TODO: This is inelegant error handling
	assert!(problem.len() == 4);
	assert!(problem[0] == "p");
	assert!(problem[1] == "cnf");
	let variables = problem[2].parse()?;
	// TODO: Check number of clauses is right
	let clauses = problem[3].parse()?;

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

	Ok((lines, variables, clauses))
}

pub fn proof_iter<P: AsRef<Path>>(
	filename: P,
) -> Result<impl Iterator<Item = Lit>> {
	Ok(BufReader::new(File::open(filename)?)
		.lines()
		.flat_map(|line| {
			line.unwrap()
				.split_whitespace()
				.map(|lit| Lit::from_dimacs_unchecked(lit.parse().unwrap()))
				// TODO: This seems nasty - can it be improved
				.collect::<Vec<_>>()
		}))
}
