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

#[cfg_attr(not(feature = "contracts"), derive(Debug))]
pub enum SatError {
	Inconsistent,
	ProofVarOutOfRange,
	FormulaVarOutOfRange,
	WrongNumberOfClauses,
	Incorrect,
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

#[predicate]
fn consistent(lits: Seq<Lit>) -> bool {
	pearlite! {
		forall<i: _, j: _> 0 <= i && i < lits.len() && 0 <= j && j < lits.len() ==>
			!lits[i].conflicts_with(lits[j])
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
	fn consistent<I: Iterator<Item = Lit>>(lits: I) -> bool {
		pearlite! {
			exists<fin: &mut I, seq: _> lits.produces(seq, *fin) &&
				fin.completed() && consistent(seq)
		}
	}

	#[predicate]
	fn some_inconsistency<I: Iterator<Item = Lit>>(lits: I) -> bool {
		pearlite! {
			exists<fin: &mut I, seq: _> lits.produces(seq, *fin) &&
				!consistent(seq)
		}
	}

	#[predicate]
	fn seq_maps_some_from(
		assignment: Seq<Option<bool>>,
		lits: Seq<Lit>,
	) -> bool {
		pearlite! {
			forall<i: _> 0 <= i && i < lits.len() ==>
			exists<p: _> assignment[lits[i].l_variable()] == Some(p) &&
				p == lits[i].l_polarity()
		}
	}

	#[predicate]
	fn maps_some_from(self, lits: Seq<Lit>) -> bool {
		pearlite! { Self::seq_maps_some_from(@self.state, lits) }
	}

	#[predicate]
	fn seq_maps_from(assignment: Seq<Option<bool>>, lits: Seq<Lit>) -> bool {
		pearlite! {
			forall<i: _> 0 <= i && i < assignment.len() ==>
				(exists<p: _> assignment[i] == Some(p)) ==
				(exists<j: _> 0 <= j && j < lits.len() &&
					(           i  ==      lits[j].l_variable()) &&
					(assignment[i] == Some(lits[j].l_polarity())))
		}
	}

	#[predicate]
	fn maps_from(self, lits: Seq<Lit>) -> bool {
		pearlite! { Self::seq_maps_from(@self.state, lits) }
	}

	#[requires(lits.invariant())]
	#[ensures(match result {
		Ok(_) => Self::vars_in_range(lits, @max_var) && Self::consistent(lits),
		Err(SatError::Inconsistent) => Self::some_inconsistency(lits),
		Err(SatError::ProofVarOutOfRange) => Self::some_var_not_in_range(lits, @max_var),
		_ => false
	})]
	#[ensures(forall<a: _> result == Ok(a) ==> (@a.state).len() == @max_var + 1)]
	#[ensures(forall<a: _> result == Ok(a) ==> iter_consumed(lits))]
	#[ensures(forall<asn: _> result == Ok(asn) ==>
		exists<fin: &mut I, seq: _> lits.produces(seq, *fin) &&
			fin.completed() && asn.maps_some_from(seq) && asn.maps_from(seq)
	)]
	fn from_unchecked_lits<I: Iterator<Item = Lit>>(
		lits: I,
		max_var: Var,
	) -> Result<Assignment, SatError> {
		let mut assignment = vec![None; max_var as usize + 1];

		#[invariant(iter_invar, iter.invariant())]
		#[invariant(assignment_len_const, (@assignment).len() == @max_var + 1)]
		#[invariant(vars_in_range, vars_in_range(produced.inner(), @max_var))]
		#[invariant(consistent, consistent(produced.inner()))]
		#[invariant(maps_some_from, Self::seq_maps_some_from(@assignment, produced.inner()))]
		#[invariant(maps_from, Self::seq_maps_from(@assignment, produced.inner()))]
		for lit in lits {
			proof_assert!(lit == produced[produced.len() - 1]);

			if !lit.in_range(max_var) {
				return Err(SatError::ProofVarOutOfRange);
			}

			let (variable, polarity) = lit.var_pol();

			if let Some(assigned_pol) = assignment[variable as usize] {
				if assigned_pol != polarity {
					proof_assert!(exists<i: _> 0 <= i && i < produced.len() && lit.conflicts_with(produced[i]));
					return Err(SatError::Inconsistent);
				}
			}

			assignment[variable as usize] = Some(polarity);
		}

		Ok(Assignment { state: assignment })
	}

	#[predicate]
	fn l_satisfies_lit(self, lit: Lit) -> bool {
		pearlite! {
			(@self.state)[lit.l_variable()] == Some(lit.l_polarity())
		}
	}

	#[predicate]
	fn l_satisfies_clause(self, clause: Seq<Lit>) -> bool {
		pearlite! {
			exists<i: _> 0 <= i && i < clause.len() &&
				self.l_satisfies_lit(clause[i])
		}
	}

	#[predicate]
	fn satisfies(self, clauses: Seq<Clause>) -> bool {
		pearlite! {
			forall<i: _> 0 <= i && i < clauses.len() ==>
				self.l_satisfies_clause(@clauses[i])
		}
	}

	#[requires(lit.l_variable() < (@self.state).len())]
	#[ensures(result == self.l_satisfies_lit(lit))]
	fn satisfies_lit(&self, lit: Lit) -> bool {
		if let Some(assigned_pol) = self.state[lit.variable() as usize] {
			if assigned_pol == lit.polarity() {
				return true;
			}
		}
		false
	}
}

pub type Clause = Vec<Lit>;

// TODO: Tidy all of these up, prove completeness and add specification of error
// meanings
#[requires(clauses.invariant())]
#[requires(proof.invariant())]
#[ensures(result == Ok(()) ==> iter_consumed(clauses) && iter_consumed(proof))]
// All proof vars in range
#[ensures(result == Ok(()) ==>
	exists<fin: &mut ProofIt, seq: _> proof.produces(seq, *fin) && fin.completed() &&
		vars_in_range(seq, @max_var)
)]
// Proof vars consistent
#[ensures(result == Ok(()) ==>
	exists<fin: &mut ProofIt, seq: _> proof.produces(seq, *fin) && fin.completed() &&
		consistent(seq)
)]
// Right number of clauses consumed
#[ensures(result == Ok(()) ==>
	exists<fin: &mut ClauseIt, seq: _> clauses.produces(seq, *fin) && fin.completed() &&
		seq.len() == @num_clauses
)]
// All clause vars in range
#[ensures(result == Ok(()) ==>
	exists<fin: &mut ClauseIt, seq: _> clauses.produces(seq, *fin) && fin.completed() &&
		forall<i: _> 0 <= i && i < seq.len() ==> vars_in_range(@seq[i], @max_var)
)]
// Soundness
#[ensures(result == Ok(()) ==>
	exists<asn: Assignment, fin: &mut ClauseIt, seq: _> clauses.produces(seq, *fin) &&
		fin.completed() && ((@asn.state).len() == @max_var + 1) && seq.len() == @num_clauses &&
			asn.satisfies(seq)
)]
pub fn check_sat<ClauseIt, ProofIt>(
	clauses: ClauseIt,
	proof: ProofIt,
	max_var: Var,
	num_clauses: usize,
) -> Result<(), SatError>
where
	ClauseIt: Iterator<Item = Clause>,
	ProofIt: Iterator<Item = Lit>,
{
	let assignment = match Assignment::from_unchecked_lits(proof, max_var) {
		Err(e) => return Err(e),
		Ok(a) => a,
	};

	let mut clauses_read = 0;

	#[invariant(iter_invar, iter.invariant())]
	#[invariant(vars_in_range, forall<i: _> 0 <= i && i < produced.len() ==> vars_in_range(@produced[i], @max_var))]
	#[invariant(clauses_read_len, produced.len() == @clauses_read && @clauses_read <= @num_clauses)]
	#[invariant(sat_status, assignment.satisfies(produced.inner()))]
	for clause in clauses {
		proof_assert!(clause == produced[produced.len() - 1]);

		let mut clause_sat = false;

		#[invariant(iter_invar, iter.invariant())]
		#[invariant(vars_in_range, vars_in_range(produced.inner(), @max_var))]
		#[invariant(sat_status, clause_sat == assignment.l_satisfies_clause(produced.inner()))]
		for lit in clause {
			proof_assert!(lit == produced[produced.len() - 1]);

			if !lit.in_range(max_var) {
				return Err(SatError::FormulaVarOutOfRange);
			}

			if !clause_sat && assignment.satisfies_lit(lit) {
				clause_sat = true;
			}
		}

		if !clause_sat {
			return Err(SatError::Incorrect);
		}

		if clauses_read >= num_clauses {
			return Err(SatError::WrongNumberOfClauses);
		}

		clauses_read += 1;
	}

	if clauses_read != num_clauses {
		return Err(SatError::WrongNumberOfClauses);
	}

	Ok(())
}
