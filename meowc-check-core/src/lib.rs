#![cfg_attr(
	not(feature = "contracts"),
	feature(stmt_expr_attributes, proc_macro_hygiene)
)]
extern crate creusot_contracts;

pub mod check_sat;

// use creusot_contracts::std::*;
// use creusot_contracts::*;

// pub struct UncheckedAsn(Vec<Lit>);

// pub enum SatResult {
//     Inconsistent,
//     VarsOutOfRange,
//     Incomplete,
//     Verified,
// }

// struct Assignment(Vec<Option<bool>>);

// struct Lit {
//     var: u32,
//     pol: bool,
// }

// struct Clause(Vec<Lit>);

// pub struct Formula {
//     clauses: Vec<Clause>,
//     num_vars: u32,
// }

// impl Lit {
//     #[predicate]
//     fn var_in_range(self, n: Int) -> bool {
//         pearlite! { @self.var < n }
//     }

//     #[predicate]
//     fn conflicts_with(self, other: Lit) -> bool {
//         pearlite! { (self.var == other.var) && (self.pol != other.pol) }
//     }

//     #[predicate]
//     fn sat_asn(self, assignment: Assignment) -> bool {
//         pearlite! { (@assignment.0)[@self.var] == Some(self.pol) }
//     }

//     #[predicate]
//     fn sat(self, assignment: UncheckedAsn) -> bool {
//         pearlite! {
//             exists<i: _> 0 <= i && i < (@assignment.0).len() &&
//                 self == (@assignment.0)[i]
//         }
//     }
// }

// impl Clause {
//     #[predicate]
//     fn vars_in_range(self, n: Int) -> bool {
//         pearlite! {
//             forall<i: _> 0 <= i && i < (@self.0).len() ==>
//                 (@self.0)[i].var_in_range(n)
//         }
//     }

//     #[predicate]
//     fn sat_asn(self, assignment: Assignment) -> bool {
//         pearlite! {
//             exists<i: _> 0 <= i && i < (@self.0).len() &&
//                 (@self.0)[i].sat_asn(assignment)
//         }
//     }

//     #[predicate]
//     fn sat(self, assignment: UncheckedAsn) -> bool {
//         pearlite! {
//             exists<i: _> 0 <= i && i < (@self.0).len() &&
//                 (@self.0)[i].sat(assignment)
//         }
//     }
// }

// impl Formula {
//     #[predicate]
//     fn invariant(self) -> bool {
//         pearlite! {
//             forall<i: _> 0 <= i && i < (@self.clauses).len() ==>
//                 (@self.clauses)[i].vars_in_range(@self.num_vars)
//         }
//     }

//     #[predicate]
//     fn sat_asn(self, assignment: Assignment) -> bool {
//         pearlite! {
//             forall<i: _> 0 <= i && i < (@self.clauses).len() ==>
//                 (@self.clauses)[i].sat_asn(assignment)
//         }
//     }

//     #[predicate]
//     fn sat(self, assignment: UncheckedAsn) -> bool {
//         pearlite! {
//             forall<i: _> 0 <= i && i < (@self.clauses).len() ==>
//                 (@self.clauses)[i].sat(assignment)
//         }
//     }
// }

// impl UncheckedAsn {
//     #[predicate]
//     fn consistent_up_to(self, idx: Int) -> bool {
//         pearlite! {
//             forall<i: _> 0 <= i && i < idx ==>
//                 forall<j: _> 0 <= j && j < i ==>
//                     !(@self.0)[i].conflicts_with((@self.0)[j])
//         }
//     }

//     #[predicate]
//     fn consistent(self) -> bool {
//         // TODO: It would be nice to define this as forall i < self.0.len()
//         // forall j < self.0.len() !i.conflicts_with(j)
//         // Maybe use lemma that conflicts_with is commutative
//         pearlite! { self.consistent_up_to((@self.0).len()) }
//     }

//     #[predicate]
//     fn vars_in_range_up_to(self, num_vars: Int, idx: Int) -> bool {
//         pearlite! {
//             forall<i: _> 0 <= i && i < idx ==>
//                 @(@self.0)[i].var < num_vars
//         }
//     }

//     #[predicate]
//     fn vars_in_range(self, num_vars: Int) -> bool {
//         pearlite! { self.vars_in_range_up_to(num_vars, (@self.0).len()) }
//     }

//     #[predicate]
//     fn maps_to_assignment_vector_up_to(
//         self,
//         assignment: Assignment,
//         idx: Int,
//     ) -> bool {
//         let values = pearlite! { @self.0 };
//         let assign = pearlite! { @assignment.0 };
//         pearlite! {
//             forall<i: _> 0 <= i && i < assign.len() ==>
//                 (exists<v: _> assign[i] == Some(v)) ==
//                 (exists<j: _> 0 <= j && j < idx &&
//                     (       i  ==     @values[j].var) &&
//                     (assign[i] == Some(values[j].pol)))
//         }
//     }

//     #[predicate]
//     fn maps_to_assignment_vector(self, assignment: Assignment) -> bool {
//         pearlite! {
//             self.maps_to_assignment_vector_up_to(assignment, (@self.0).len())
//         }
//     }

//     #[predicate]
//     fn maps_to_some_up_to(self, assignment: Assignment, idx: Int) -> bool {
//         pearlite! {
//             forall<i: _> 0 <= i && i < idx ==> exists<v: _>
//                 (@assignment.0)[@(@self.0)[i].var] == Some(v) &&
//                 v == (@self.0)[i].pol
//         }
//     }

//     #[predicate]
//     fn maps_to_some(self, assignment: Assignment) -> bool {
//         pearlite! {
//             self.maps_to_some_up_to(assignment, (@self.0).len())
//         }
//     }

//     #[ensures(match result {
//         Ok(_)                          => { self.vars_in_range(@num_vars) && self.consistent() }
//         Err(SatResult::Inconsistent)   => { !self.consistent() }
//         Err(SatResult::VarsOutOfRange) => { !self.vars_in_range(@num_vars) }
//         _                              => false
//     })]
//     #[ensures(forall<a: _> result == Ok(a) ==> (@a.0).len() == @num_vars)]
//     #[ensures(forall<a: _> result == Ok(a) ==> self.maps_to_assignment_vector(a))]
//     #[ensures(forall<a: _> result == Ok(a) ==> self.maps_to_some(a))]
//     fn to_assignment_vec(
//         &self,
//         num_vars: u32,
//     ) -> Result<Assignment, SatResult> {
//         let mut assignment = vec![None; num_vars as usize];

//         #[invariant(assignment_len_const, (@assignment).len() == @num_vars)]
//         #[invariant(iter_bounded, produced.len() <= (@self.0).len())]
//         #[invariant(consistent, self.consistent_up_to(produced.len()))]
//         #[invariant(vars_in_range, self.vars_in_range_up_to(@num_vars, produced.len()))]
//         #[invariant(mappings_valid, self.maps_to_assignment_vector_up_to(Assignment(assignment), produced.len()))]
//         #[invariant(mappings_valid_2, self.maps_to_some_up_to(Assignment(assignment), produced.len()))]
//         for lit in self.0.iter() {
//             proof_assert!(*lit == (@self.0)[produced.len() - 1]);

//             if lit.var >= num_vars {
//                 return Err(SatResult::VarsOutOfRange);
//             }

//             if let Some(assigned_pol) = assignment[lit.var as usize] {
//                 if assigned_pol != lit.pol {
//                     return Err(SatResult::Inconsistent);
//                 }
//             }

//             assignment[lit.var as usize] = Some(lit.pol);
//         }

//         Ok(Assignment(assignment))
//     }
// }

// impl Formula {
//     #[maintains(self.invariant())]
//     #[ensures(match result {
//         SatResult::Inconsistent   => { !input_assignment.consistent() }
//         SatResult::VarsOutOfRange => { !input_assignment.vars_in_range(@self.num_vars) }
//         _                         => { input_assignment.consistent() && input_assignment.vars_in_range(@self.num_vars) }
//     })]
//     #[ensures(result == SatResult::Verified   ==> self.sat(*input_assignment))]
//     #[ensures(result == SatResult::Incomplete ==> !self.sat(*input_assignment))]
//     pub fn check_sat(&self, input_assignment: &UncheckedAsn) -> SatResult {
//         match input_assignment.to_assignment_vec(self.num_vars) {
//             Err(e) => e,
//             Ok(assignment) => {
//                 let res = self.is_sat(&assignment);
//                 proof_assert!(res == SatResult::Verified ==> self.sat(*input_assignment));
//                 res
//             }
//         }
//     }

//     #[requires((@assignment.0).len() == @self.num_vars)]
//     #[requires(self.invariant())]
//     #[ensures(match result {
//         SatResult::Verified   => {  self.sat_asn(*assignment) }
//         SatResult::Incomplete => { !self.sat_asn(*assignment) }
//         _                    => false
//     })]
//     fn is_sat(&self, assignment: &Assignment) -> SatResult {
//         #[invariant(iter_bounded, produced.len() <= (@self.clauses).len())]
//         #[invariant(prev_clauses_sat,
//             forall<j: _> 0 <= j && j < produced.len() ==>
//                 (@self.clauses)[j].sat_asn(*assignment)
//         )]
//         for clause in self.clauses.iter() {
//             proof_assert!(*clause == (@self.clauses)[produced.len() - 1]);

//             let mut clause_sat = false;

//             #[invariant(iter_bounded, produced.len() <= (@clause.0).len())]
//             #[invariant(clause_not_sat_yet,
//                 forall<i: _> 0 <= i && i < produced.len() ==>
//                     !(@clause.0)[i].sat_asn(*assignment)
//             )]
//             for lit in clause.0.iter() {
//                 proof_assert!(*lit == (@clause.0)[produced.len() - 1]);

//                 if let Some(assigned_pol) = assignment.0[lit.var as usize] {
//                     if assigned_pol == lit.pol {
//                         clause_sat = true;
//                         break;
//                     }
//                 }
//             }

//             if !clause_sat {
//                 return SatResult::Incomplete;
//             }
//         }

//         return SatResult::Verified;
//     }
// }
