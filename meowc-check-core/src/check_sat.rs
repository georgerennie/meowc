extern crate creusot_contracts;
use creusot_contracts::{
	ensures, invariant, logic, predicate, proof_assert, requires, DeepModel,
	ShallowModel,
};

#[cfg(feature = "contracts")]
use creusot_contracts::{
	ghost, invariant::Invariant, pearlite, Int, Iterator, Seq,
};

#[predicate]
fn iter_consumed<I: Iterator>(iter: I) -> bool {
	pearlite! {
		exists<fin: &mut I, seq: _>
			iter.produces(seq, *fin) && fin.completed()
	}
}

pub enum SatResult {
	Inconsistent,
	VarOutOfRange,
	Incorrect,
	Verified,
}

pub type Var = u32;
pub type RawLit = i32;

#[derive(creusot_contracts::PartialEq, Eq, creusot_contracts::Clone, Copy)]
pub struct Lit {
	lit: RawLit,
}

impl Lit {
	#[logic]
	fn l_variable(self) -> Int {
		pearlite! { (@self.lit).abs_diff(0) }
	}

	#[logic]
	fn l_polarity(self) -> bool {
		pearlite! { (@self.lit) >= 0 }
	}

	#[predicate]
	fn l_in_range(self, max_var: Int) -> bool {
		pearlite! { self.l_variable() <= max_var }
	}

	#[predicate]
	fn conflicts_with(self, other: Lit) -> bool {
		self.l_variable() == other.l_variable()
			&& self.l_polarity() != other.l_polarity()
	}

	#[ensures(@result == self.l_variable())]
	pub fn variable(&self) -> Var {
		// TODO: This function just implements i32::unsigned_abs
		// Look into adding this to creusot
		if self.lit == RawLit::MIN {
			0x80000000
		} else if self.lit < 0 {
			-self.lit as Var
		} else {
			self.lit as Var
		}
	}

	#[ensures(result == self.l_polarity())]
	fn polarity(&self) -> bool {
		self.lit >= 0
	}

	#[ensures(@result.0 == self.l_variable())]
	#[ensures(result.1 == self.l_polarity())]
	fn var_pol(&self) -> (Var, bool) {
		(self.variable(), self.polarity())
	}

	#[ensures(result.lit == l)]
	pub fn from_dimacs_unchecked(l: RawLit) -> Lit {
		Self { lit: l }
	}

	#[ensures(result == self.l_in_range(@max_var))]
	fn in_range(&self, max_var: Var) -> bool {
		self.variable() <= max_var
	}
}

impl ShallowModel for Lit {
	type ShallowModelTy = Lit;

	#[logic]
	fn shallow_model(self) -> Self {
		self
	}
}

impl DeepModel for Lit {
	type DeepModelTy = Lit;

	#[logic]
	fn deep_model(self) -> Self {
		self
	}
}

struct Assignment {
	state: Vec<Option<bool>>,
}

#[predicate]
fn vars_in_range(lits: Seq<Lit>, max_var: Int) -> bool {
	pearlite! {
		forall<i: _> 0 <= i && i < lits.len() ==> lits[i].l_in_range(max_var)
	}
}

impl Assignment {
	#[predicate]
	fn vars_in_range<I: Iterator<Item = Lit>>(lits: I, max_var: Int) -> bool {
		pearlite! {
			exists<fin: &mut I, seq: _> lits.produces(seq, *fin) &&
				fin.completed() && vars_in_range(seq, max_var)
		}
	}

	#[predicate]
	fn some_var_not_in_range<I: Iterator<Item = Lit>>(
		lits: I,
		max_var: Int,
	) -> bool {
		pearlite! {
			exists<fin: &mut I, seq: _> lits.produces(seq, *fin) &&
				!vars_in_range(seq, max_var)
		}
	}

	#[predicate]
	fn consistent(lits: Seq<Lit>) -> bool {
		pearlite! {
			// TODO: change j < i to j < lits.len() && j != i
			forall<i: _, j: _> 0 <= i && i < lits.len() && 0 <= j && j < i ==>
				!lits[i].conflicts_with(lits[j])
		}
	}

	#[predicate]
	fn maps_to_some(lits: Seq<Lit>, assignment: Seq<Option<bool>>) -> bool {
		pearlite! {
			forall<i: _> 0 <= i && i < lits.len() ==>
			exists<v: _>
				assignment[lits[i].l_variable()] == Some(v) &&
				v == lits[i].l_polarity()
		}
	}

	#[requires(lits.invariant())]
	#[ensures(match result {
		Ok(_) => Self::vars_in_range(lits, @max_var),
		Err(SatResult::Inconsistent) => true,
		Err(SatResult::VarOutOfRange) => Self::some_var_not_in_range(lits, @max_var),
		_ => false
	})]
	#[ensures(forall<a: _> result == Ok(a) ==> (@a.state).len() == @max_var + 1)]
	#[ensures(forall<a: _> result == Ok(a) ==> iter_consumed(lits))]
	#[ensures(forall<a: _> result == Ok(a) ==>
		exists<fin: &mut I, seq: _> lits.produces(seq, *fin) &&
			fin.completed() && Self::maps_to_some(seq, @a.state)
	)]
	fn from_unchecked_lits<I: Iterator<Item = Lit>>(
		lits: I,
		max_var: Var,
	) -> Result<Assignment, SatResult> {
		let mut assignment = vec![None; max_var as usize + 1];

		#[invariant(iter_invar, iter.invariant())]
		#[invariant(assignment_len_const, (@assignment).len() == @max_var + 1)]
		#[invariant(vars_in_range, vars_in_range(produced.inner(), @max_var))]
		#[invariant(maps_to_some, Self::maps_to_some(produced.inner(), @assignment))]
		for lit in lits {
			if !lit.in_range(max_var) {
				proof_assert!(!produced[produced.len() - 1].l_in_range(@max_var));
				return Err(SatResult::VarOutOfRange);
			}

			let (variable, polarity) = lit.var_pol();

			if let Some(assigned_pol) = assignment[variable as usize] {
				if assigned_pol != polarity {
					return Err(SatResult::Inconsistent);
				}
			}

			assignment[variable as usize] = Some(polarity);
		}

		Ok(Assignment { state: assignment })
	}

	#[requires(lit.l_variable() < (@self.state).len())]
	#[ensures(result ==
		((@self.state)[lit.l_variable()] == Some(lit.l_polarity()))
	)]
	fn satisfies(&self, lit: Lit) -> bool {
		if let Some(assigned_pol) = self.state[lit.variable() as usize] {
			if assigned_pol == lit.polarity() {
				return true;
			}
		}
		false
	}
}

pub type Clause = Vec<Lit>;

// TODO: Check that the right number of clauses are read
#[requires(clauses.invariant())]
#[requires(proof.invariant())]
#[ensures(result == SatResult::Verified ==> iter_consumed(clauses) && iter_consumed(proof))]
#[ensures(result == SatResult::Verified ==>
	exists<fin: &mut ProofIt, seq: _> proof.produces(seq, *fin) && fin.completed() &&
		vars_in_range(seq, @max_var)
)]
#[ensures(result == SatResult::Verified ==>
	exists<fin: &mut ClauseIt, seq: _> clauses.produces(seq, *fin) && fin.completed() &&
		forall<i: _> 0 <= i && i < seq.len() ==> vars_in_range(@seq[i], @max_var)
)]
pub fn check_sat<ClauseIt, ProofIt>(
	clauses: ClauseIt,
	proof: ProofIt,
	max_var: Var,
) -> SatResult
where
	ClauseIt: Iterator<Item = Clause>,
	ProofIt: Iterator<Item = Lit>,
{
	let assignment = match Assignment::from_unchecked_lits(proof, max_var) {
		Err(err) => return err,
		Ok(a) => a,
	};

	#[invariant(iter_invar, iter.invariant())]
	#[invariant(vars_in_range, forall<i: _> 0 <= i && i < produced.len() ==> vars_in_range(@produced[i], @max_var))]
	for clause in clauses {
		let mut clause_sat = false;

		#[invariant(iter_invar, iter.invariant())]
		#[invariant(vars_in_range, vars_in_range(produced.inner(), @max_var))]
		for lit in clause {
			if !lit.in_range(max_var) {
				return SatResult::VarOutOfRange;
			}

			if !clause_sat && assignment.satisfies(lit) {
				clause_sat = true;
			}
		}

		if !clause_sat {
			return SatResult::Incorrect;
		}
	}

	SatResult::Verified
}
