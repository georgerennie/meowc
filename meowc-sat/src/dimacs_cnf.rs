use crate::{
	solver::Solver,
	types::{Clause, Lit},
};
use pest::{error::Error, Parser};

#[derive(Parser)]
#[grammar = "dimacs_cnf.pest"]
struct DIMACSParser;

pub fn parse_dimacs(dimacs_str: &str) -> Result<Solver, Error<Rule>> {
	let dimacs = DIMACSParser::parse(Rule::dimacs, dimacs_str)?
		.next()
		.unwrap();

	let mut lines = dimacs
		.into_inner()
		.skip_while(|line| line.as_rule() == Rule::comment);

	let num_vars = lines
		.next()
		.unwrap()
		.into_inner()
		.skip_while(|part| part.as_rule() != Rule::num_variables)
		.next()
		.unwrap()
		.as_str()
		.parse::<u32>()
		.unwrap();

	let mut solver = Solver::new(num_vars);

	for clause in lines.filter(|line| line.as_rule() == Rule::clause) {
		solver.add_clause(
			&clause
				.into_inner()
				.map(|lit| Lit::from(lit.as_str().parse::<i32>().unwrap()))
				.collect::<Clause>(),
		);
	}

	Ok(solver)
}
