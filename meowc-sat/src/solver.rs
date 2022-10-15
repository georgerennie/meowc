use crate::{
	luby::Luby,
	types::{Clause, ClauseId, DecisionLevel, Lit, SatResult, VarId},
};

const RESTART_SCALE: u64 = 512;

#[derive(Clone)]
pub struct Solver {
	clauses: Vec<Clause>,

	decision_level: DecisionLevel,
	num_vars: u32,
	num_assigned: u32,
	conflicts: u64,
	next_restart: u64,
	restarts: u64,

	assignments: Vec<Option<bool>>,
	decision_levels: Vec<DecisionLevel>,
	antecedents: Vec<ClauseId>,

	phases: Vec<bool>,

	frequencies: Vec<i32>,
	frequencies_cache: Vec<i32>,

	luby: Luby,
}

impl Solver {
	pub fn new(num_vars: u32) -> Self {
		// This accounts for index 0 being unused for simplicity
		let num_vars = num_vars + 1;

		Self {
			clauses: vec![],

			decision_level: 0,
			num_vars,
			num_assigned: 0,
			conflicts: 0,
			next_restart: RESTART_SCALE,
			restarts: 0,

			assignments: vec![None; num_vars as usize],
			decision_levels: vec![0; num_vars as usize],
			antecedents: vec![-1; num_vars as usize],

			phases: vec![false; num_vars as usize],
			frequencies: vec![0; num_vars as usize],
			frequencies_cache: vec![0; num_vars as usize],

			luby: Default::default(),
		}
	}

	#[inline]
	fn all_assigned(&self) -> bool {
		self.num_assigned == self.num_vars
	}

	pub fn solve(&mut self) -> SatResult {
		self.decision_level = 0;

		if let Err(_) = self.unit_propagate() {
			return SatResult::Unsat;
		}

		while !self.all_assigned() {
			self.decision_level += 1;

			self.assign(self.choose_assignment(), -1);

			while let Err(conflict_clause) = self.unit_propagate() {
				if self.decision_level == 0 {
					return SatResult::Unsat;
				}

				self.conflict_analysis(conflict_clause);
				self.conflicts += 1;

				if self.should_restart() {
					self.restarts += 1;
					self.backtrack(0);
				}
			}
		}

		SatResult::Sat
	}

	pub fn print_stats(&self) {
		println!("{} final clauses", self.clauses.len());
		println!("{} conflicts", self.conflicts);
		println!("{} restarts", self.restarts);
	}

	pub fn add_clause(&mut self, clause: &Clause) {
		for lit in clause.iter() {
			let var = lit.var();

			if self.frequencies[var] != -1 {
				self.frequencies[var] += 1;
			}
			self.frequencies_cache[var] += 1;
		}

		self.clauses.push(clause.to_vec());
	}

	fn should_restart(&mut self) -> bool {
		let should_restart = self.conflicts >= self.next_restart;
		if should_restart {
			self.next_restart =
				self.conflicts + self.luby.next() * RESTART_SCALE;
		}

		should_restart
	}

	fn unit_propagate(&mut self) -> Result<(), ClauseId> {
		'outer_loop: loop {
			'clause_loop: for (clause_id, clause) in
				self.clauses.iter().enumerate()
			{
				let mut unassigned_lit = None;

				for lit in clause.iter() {
					match self.assignments[lit.var()] {
						// If literal is sat clause is sat so go to next clause
						Some(assignment) if assignment == lit.as_bool() => {
							continue 'clause_loop
						}

						// If more than one unassigned var we cant do anything
						// so go to next clause, else save unassigned var
						None => match unassigned_lit {
							Some(_) => continue 'clause_loop,
							None => unassigned_lit = Some(*lit),
						},

						_ => (),
					}
				}

				// All clauses still considered here have either 0 or 1
				// unassigned vars and the rest unsatisfied by the assignment

				// Unsat
				if unassigned_lit.is_none() {
					return Err(clause_id as ClauseId);
				}

				// Unit clause
				self.assign(unassigned_lit.unwrap(), clause_id as ClauseId);
				continue 'outer_loop;
			}

			// No unit clauses found - unit clauses cause the outer_loop to be
			// continued early
			return Ok(());
		}
	}

	fn assign(&mut self, lit: Lit, antecedent: ClauseId) {
		let var = lit.var();
		self.assignments[var] = Some(lit.as_bool());
		self.decision_levels[var] = self.decision_level;
		self.antecedents[var] = antecedent;
		self.frequencies[var] = -1;
		self.num_assigned += 1;
	}

	fn unassign(&mut self, var: VarId) {
		if let Some(assignment) = self.assignments[var] {
			self.phases[var] = assignment;
		}

		self.assignments[var] = None;
		self.antecedents[var] = -1;
		self.frequencies[var] = self.frequencies_cache[var];
		self.num_assigned -= 1;
	}

	fn conflict_analysis(&mut self, conflict_id: ClauseId) {
		let learnt_clause = self.derive_1uip_clause(conflict_id);

		self.add_clause(&learnt_clause);

		// Find greatest decision level below conflict decision level that
		// assigns to the learnt clause for backgracking
		self.backtrack(
			learnt_clause
				.iter()
				.map(|lit| self.decision_levels[lit.var() as usize])
				.filter(|&level| level < self.decision_level)
				.max()
				.unwrap_or(0),
		);
	}

	/// Backtracks to self.decision_level
	fn backtrack(&mut self, backtrack_level: DecisionLevel) {
		// TODO: Can this be iterators?
		for var in 0..self.decision_levels.len() {
			if self.decision_levels[var] > backtrack_level {
				self.unassign(var);
			}
		}

		self.decision_level = backtrack_level;
	}

	fn derive_1uip_clause(&mut self, conflict_id: ClauseId) -> Clause {
		let mut learnt_clause = self.clauses[conflict_id as usize].clone();

		loop {
			let mut conflict_level_lits: u32 = 0;
			let mut resolvent_lit = None;

			for lit in learnt_clause.iter() {
				let var = lit.var();

				// Only consider literals assigned at the conflict decicision
				// level
				if self.decision_levels[var] != self.decision_level {
					continue;
				}

				conflict_level_lits += 1;

				// If literal at this level has antecedent, save for resolving
				if self.antecedents[var] != -1 {
					resolvent_lit = Some(*lit);
				}
			}

			// One lit at the conflict level means that is a UIP
			if conflict_level_lits == 1 {
				break;
			}

			debug_assert!(resolvent_lit.is_some());
			learnt_clause = self.resolve(&learnt_clause, unsafe {
				resolvent_lit.unwrap_unchecked()
			});
		}

		learnt_clause
	}

	fn resolve(&self, clause: &Clause, resolvent: Lit) -> Clause {
		let var = resolvent.var();
		let antecedent = &self.clauses[self.antecedents[var] as usize];

		// TODO: This isnt modifying in place, is this an issue
		// Join clauses and remove resolvent
		let mut new_clause: Clause = clause
			.iter()
			.chain(antecedent.iter())
			.filter(|lit| lit.var() != var)
			.cloned()
			.collect();

		// Remove duplicates
		new_clause.sort();
		new_clause.dedup();

		new_clause
	}

	fn choose_assignment(&self) -> Lit {
		// TODO: Make this a better scheme, atm it just picks the max freq
		// unassigned var

		let var: VarId = self
			.frequencies
			.iter()
			.enumerate()
			.max_by_key(|&(_, item)| item)
			.unwrap()
			.0;

		Lit::from((var, self.phases[var]))
	}
}
