use serde::Deserialize;
use std::{fmt, sync::Arc};

/// The type representing time instants.
pub type Time = usize;

/// The type of indexes of propositional variables.
pub type Idx = u8;

/// A formula represented via its syntax tree.
/// This is a recursive data structure, so it requires the use of smart pointers.
/// We use `Arc` to make it compatible with parallel computations.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub enum LTLformula {
    Atom(bool, Idx),
    Not(Arc<LTLformula>),
    Next(Arc<LTLformula>),
    Globally(Arc<LTLformula>),
    Finally(Arc<LTLformula>),
    And(Arc<LTLformula>, Arc<LTLformula>),
    Or(Arc<LTLformula>, Arc<LTLformula>),
    // Implies(Arc<LTLformula>, Arc<LTLformula>),
    Until(Arc<LTLformula>, Arc<LTLformula>),
    Release(Arc<LTLformula>, Arc<LTLformula>),
}

impl fmt::Display for LTLformula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LTLformula::Atom(true, var) => write!(f, "x{}", var),
            LTLformula::Atom(false, var) => write!(f, "¬x{}", var),
            LTLformula::Not(subformula) => write!(f, "¬({})", subformula),
            LTLformula::Next(subformula) => write!(f, "X({})", subformula),
            LTLformula::Globally(subformula) => write!(f, "G({})", subformula),
            LTLformula::Finally(subformula) => write!(f, "F({})", subformula),
            LTLformula::And(left_subformula, right_subformula) => {
                write!(f, "({})∧({})", left_subformula, right_subformula)
            }
            LTLformula::Or(left_subformula, right_subformula) => {
                write!(f, "({})∨({})", left_subformula, right_subformula)
            }
            // LTLformula::Implies(left_subformula, right_subformula) => {
            //     write!(f, "({})→({})", left_subformula, right_subformula)
            // }
            LTLformula::Until(left_subformula, right_subformula) => {
                write!(f, "({})U({})", left_subformula, right_subformula)
            }
            LTLformula::Release(left_subformula, right_subformula) => {
                write!(f, "({})R({})", left_subformula, right_subformula)
            }
        }
    }
}

impl LTLformula {
    pub fn print_w_named_vars(&self, vars: &[String]) -> String {
        match self {
            LTLformula::Atom(true, var) => vars[*var as usize].clone(),
            LTLformula::Atom(false, var) => format!("¬({})", vars[*var as usize]),
            LTLformula::Not(subformula) => format!("¬({})", subformula.print_w_named_vars(vars)),
            LTLformula::Next(subformula) => format!("X({})", subformula.print_w_named_vars(vars)),
            LTLformula::Globally(subformula) => {
                format!("G({})", subformula.print_w_named_vars(vars))
            }
            LTLformula::Finally(subformula) => {
                format!("F({})", subformula.print_w_named_vars(vars))
            }
            LTLformula::And(left_subformula, right_subformula) => {
                format!(
                    "({})∧({})",
                    left_subformula.print_w_named_vars(vars),
                    right_subformula.print_w_named_vars(vars)
                )
            }
            LTLformula::Or(left_subformula, right_subformula) => {
                format!(
                    "({})∨({})",
                    left_subformula.print_w_named_vars(vars),
                    right_subformula.print_w_named_vars(vars)
                )
            }
            // LTLformula::Implies(left_subformula, right_subformula) => {
            //     format!(
            //         "({})→({})",
            //         left_subformula.print_w_named_vars(vars),
            //         right_subformula.print_w_named_vars(vars)
            //     )
            // }
            LTLformula::Until(left_subformula, right_subformula) => {
                format!(
                    "({})U({})",
                    left_subformula.print_w_named_vars(vars),
                    right_subformula.print_w_named_vars(vars)
                )
            }
            LTLformula::Release(left_subformula, right_subformula) => {
                format!(
                    "({})R({})",
                    left_subformula.print_w_named_vars(vars),
                    right_subformula.print_w_named_vars(vars)
                )
            }
        }
    }

    /// Returns the highest propositional variable index appearing in the formula, plus 1.
    /// Used to count how many variables are needed to interpret the formula.
    pub fn vars(&self) -> Idx {
        match self {
            LTLformula::Atom(_, n) => *n + 1,
            LTLformula::Not(subformula)
            | LTLformula::Next(subformula)
            | LTLformula::Globally(subformula)
            | LTLformula::Finally(subformula) => subformula.as_ref().vars(),
            LTLformula::And(left_subformula, right_subformula)
            | LTLformula::Or(left_subformula, right_subformula)
            // | LTLformula::Implies(left_subformula, right_subformula)
            | LTLformula::Release(left_subformula, right_subformula)
            | LTLformula::Until(left_subformula, right_subformula) => {
                left_subformula.vars().max(right_subformula.vars())
            }
        }
    }

    /// Evaluate a formula on a trace.
    pub fn eval<const N: usize>(&self, trace: &[[bool; N]]) -> bool {
        match self {
            LTLformula::Atom(pos, var) => {
                !trace.is_empty()
                    && *trace
                        .first()
                        .expect("interpret atomic proposition in trace")
                        .get(*var as usize)
                        .expect("interpret atomic proposition in trace")
                        == *pos
            }
            LTLformula::Not(subformula) => !subformula.eval(trace),
            LTLformula::Next(subformula) => !trace.is_empty() && subformula.eval(&trace[1..]),
            // Globally is interpreted by reverse temporal order because interpreting on shorter traces is generally faster.
            LTLformula::Globally(subformula) => {
                (0..trace.len()).rev().all(|t| subformula.eval(&trace[t..]))
            }
            // Finally is interpreted by reverse temporal order because interpreting on shorter traces is generally faster.
            LTLformula::Finally(subformula) => {
                (0..trace.len()).rev().any(|t| subformula.eval(&trace[t..]))
            }
            LTLformula::And(left_subformula, right_subformula) => {
                left_subformula.eval(trace) && right_subformula.eval(trace)
            }
            LTLformula::Or(left_subformula, right_subformula) => {
                left_subformula.eval(trace) || right_subformula.eval(trace)
            }
            // LTLformula::Implies(left_subformula, right_subformula) => {
            //     !left_subformula.eval(trace) || right_subformula.eval(trace)
            // }
            LTLformula::Until(left_subformula, right_subformula) => {
                for t in 0..trace.len() {
                    let t_trace = &trace[t..];
                    if right_subformula.eval(t_trace) {
                        return true;
                    } else if !left_subformula.eval(t_trace) {
                        return false;
                    }
                }
                // Until is not satisfied if its right-hand-side argument never becomes true.
                false
            }
            LTLformula::Release(left_subformula, right_subformula) => {
                for t in 0..trace.len() {
                    let t_trace = &trace[t..];
                    if !right_subformula.eval(t_trace) {
                        return false;
                    } else if left_subformula.eval(t_trace) {
                        return true;
                    }
                }
                true
            }
        }
    }
}

#[cfg(test)]
mod eval {
    use super::*;

    const ATOM_0: LTLformula = LTLformula::Atom(true, 0);

    const ATOM_1: LTLformula = LTLformula::Atom(true, 1);

    #[test]
    fn atomic_prop() {
        let trace = [[true]];
        assert!(ATOM_0.eval(&trace));

        let trace = [[false]];
        assert!(!ATOM_0.eval(&trace));
    }

    #[test]
    fn not() {
        let formula = LTLformula::Not(Arc::new(ATOM_0));

        let trace = [[false]];
        assert!(formula.eval(&trace));

        let trace = [[true]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn next() {
        let formula = LTLformula::Next(Arc::new(ATOM_0));

        let trace = [[false], [true]];
        assert!(formula.eval(&trace));

        let trace = [[true], [false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn globally() {
        let formula = LTLformula::Globally(Arc::new(ATOM_0));

        let trace = [[true], [true], [true]];
        assert!(formula.eval(&trace));

        let trace = [[true], [false], [true]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn and() {
        let formula = LTLformula::And(Arc::new(ATOM_0), Arc::new(ATOM_1));

        let trace = [[true, true]];
        assert!(formula.eval(&trace));

        let trace = [[true, false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn or() {
        let formula = LTLformula::Or(Arc::new(ATOM_0), Arc::new(ATOM_1));

        let trace = [[true, false]];
        assert!(formula.eval(&trace));

        let trace = [[false, false]];
        assert!(!formula.eval(&trace));
    }

    #[test]
    fn until() {
        let formula = LTLformula::Until(Arc::new(ATOM_0), Arc::new(ATOM_1));

        let trace = [[true, false], [false, true], [false, false]];
        assert!(formula.eval(&trace));

        let trace = [[true, false], [true, false], [false, false]];
        assert!(!formula.eval(&trace));

        // Until is not satisfied if its right-hand-side argument never becomes true.
        let trace = [[true, false], [true, false], [true, false]];
        assert!(!formula.eval(&trace));
    }
}
