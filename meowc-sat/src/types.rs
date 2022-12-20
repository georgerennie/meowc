use std::{fmt, num::NonZeroU32, ops::Not};

pub type VarId = usize;
pub type Clause = Vec<Lit>;
pub type ClauseId = i32;
pub type DecisionLevel = u32;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SatResult {
	Sat,
	Unsat,
}

impl fmt::Display for SatResult {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				SatResult::Sat => "SATISFIABLE",
				SatResult::Unsat => "UNSATISFIABLE",
			}
		)
	}
}

/// Literal encoded in u32 such that n in DIMACS is (n << 1) + 1 and -n in
/// DIMACS is (n << 1)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lit(NonZeroU32);

impl Lit {
	#[inline]
	pub fn var(&self) -> VarId {
		(self.0.get() >> 1) as usize
	}

	#[inline]
	pub fn as_bool(&self) -> bool {
		(*self).into()
	}
}

impl Not for Lit {
	type Output = Self;

	#[inline]
	fn not(self) -> Self {
		Self(unsafe { NonZeroU32::new_unchecked(self.0.get() ^ 1) })
	}
}

impl From<Lit> for bool {
	#[inline]
	fn from(lit: Lit) -> Self {
		(lit.0.get() & 1) != 0
	}
}

impl From<(VarId, bool)> for Lit {
	#[inline]
	fn from((var, b): (VarId, bool)) -> Self {
		Self(unsafe {
			NonZeroU32::new_unchecked(((var as u32) << 1) | (b as u32))
		})
	}
}

impl From<Lit> for (VarId, bool) {
	#[inline]
	fn from(lit: Lit) -> Self {
		(lit.var(), lit.into())
	}
}

impl From<i32> for Lit {
	#[inline]
	fn from(i: i32) -> Self {
		assert_ne!(i, 0);

		Self(unsafe {
			NonZeroU32::new_unchecked(
				(if i < 0 { -i << 1 } else { (i << 1) + 1 }) as u32,
			)
		})
	}
}

impl From<Lit> for i32 {
	#[inline]
	fn from(lit: Lit) -> Self {
		if lit.0.get() % 2 == 0 {
			-(lit.var() as i32)
		} else {
			lit.var() as i32
		}
	}
}

impl fmt::Debug for Lit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", i32::from(*self))
	}
}

impl fmt::Display for Lit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", i32::from(*self))
	}
}
